#!/usr/bin/env node

/**
 * Auto-migration script
 * Creates the database if it doesn't exist and runs all migrations
 * This script:
 * 1. Creates the 't3chat' database if it doesn't exist
 * 2. Applies all pending Drizzle migrations
 * 3. Creates tables and indexes automatically
 */

import postgres from 'postgres';
import { drizzle } from 'drizzle-orm/postgres-js';
import { migrate } from 'drizzle-orm/postgres-js/migrator';
import 'dotenv/config';

const DATABASE_NAME = 't3chat';

console.log('Starting auto-migration...\n');

// Helper function to safely execute SQL with error handling
const safeExecute = async (description, operation) => {
  try {
    await operation();
    console.log(`   ${description}`);
    return true;
  } catch (error) {
    console.error(`   ${description}: ${error.message}`);
    throw error;
  }
};

// Parse connection string and extract components
const parseConnectionString = (connStr) => {
  const url = new URL(connStr);
  return {
    user: url.username || 'postgres',
    password: url.password || 'postgres',
    host: url.hostname || 'localhost',
    port: parseInt(url.port) || 5432,
    database: url.pathname.slice(1) || 'postgres',
  };
};

// Build connection string
const buildConnectionString = (params) => {
  return `postgresql://${params.user}:${params.password}@${params.host}:${params.port}/${params.database}`;
};

const autoMigrate = async () => {
  const originalConnectionString = process.env.DATABASE_URL;
  
  if (!originalConnectionString) {
    console.error('DATABASE_URL not found in environment variables');
    process.exit(1);
  }

  const params = parseConnectionString(originalConnectionString);
  
  // Step 1: Connect to default postgres database to create t3chat database
  console.log('Step 1: Creating database if needed...');
  const adminConnectionString = buildConnectionString({
    ...params,
    database: 'postgres', // Connect to default postgres database
  });

  let adminSql;
  try {
    adminSql = postgres(adminConnectionString, { 
      prepare: false,
      max: 1,
    });

    // Check if database exists
    const dbCheckResult = await adminSql`
      SELECT 1 FROM pg_database WHERE datname = ${DATABASE_NAME}
    `;

    if (dbCheckResult.length === 0) {
      // Database doesn't exist, create it
      await safeExecute(
        `Created database "${DATABASE_NAME}"`,
        async () => {
          // postgres-js doesn't support CREATE DATABASE in prepared statements
          // We need to execute it directly
          await adminSql.unsafe(`CREATE DATABASE ${DATABASE_NAME}`);
        }
      );
    } else {
      console.log(`   Database "${DATABASE_NAME}" already exists`);
    }
  } catch (error) {
    console.error(`Failed to create database: ${error.message}`);
    throw error;
  } finally {
    if (adminSql) {
      await adminSql.end();
    }
  }

  // Step 2: Connect to t3chat database and run migrations
  console.log('\nStep 2: Running migrations...');
  
  const targetConnectionString = buildConnectionString({
    ...params,
    database: DATABASE_NAME,
  });

  let migrationSql;
  try {
    migrationSql = postgres(targetConnectionString, { 
      prepare: false,
      max: 1,
    });

    const db = drizzle(migrationSql);

    await safeExecute(
      'Applied all migrations',
      async () => {
        await migrate(db, { migrationsFolder: './drizzle' });
      }
    );

    console.log('\nAuto-migration completed successfully!\n');
    console.log('Database Summary:');
    console.log(`   • Database: ${DATABASE_NAME}`);
    console.log(`   • Schema: public (default)`);
    console.log(`   • All tables and indexes created`);
    console.log(`   • Connection: ${params.host}:${params.port}\n`);

  } catch (error) {
    console.error('\nMigration failed:', error.message);
    console.error('\nPossible issues:');
    console.error('   • Database permissions');
    console.error('   • Connection issues');
    console.error('   • Migration files are missing or corrupted');
    console.error('   • Database is already at the latest version\n');
    throw error;
  } finally {
    if (migrationSql) {
      await migrationSql.end();
    }
  }
};

// Run the migration
autoMigrate().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});

