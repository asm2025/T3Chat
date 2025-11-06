#!/usr/bin/env node

/**
 * Test script to verify migration setup
 * This script checks if the database and tables are set up correctly
 */

import postgres from 'postgres';
import 'dotenv/config';

const DATABASE_NAME = 't3chat';

console.log('Testing migration setup...\n');

const testMigration = async () => {
  const connectionString = process.env.DATABASE_URL;
  
  if (!connectionString) {
    console.error('DATABASE_URL not found in environment variables');
    process.exit(1);
  }

  console.log('Connection string:', connectionString.replace(/:[^:@]+@/, ':***@'));

  let sql;
  try {
    sql = postgres(connectionString, { 
      prepare: false,
      max: 1,
    });

    // Test 1: Check database connection
    console.log('\n1. Testing database connection...');
    await sql`SELECT 1 as test`;
    console.log('   Database connection successful');

    // Test 2: Check if we're connected to the right database
    console.log('\n2. Checking database name...');
    const dbResult = await sql`SELECT current_database() as db`;
    const currentDb = dbResult[0].db;
    console.log(`   Connected to database: ${currentDb}`);
    
    if (currentDb !== DATABASE_NAME) {
      console.log(`   Warning: Connected to "${currentDb}" but expected "${DATABASE_NAME}"`);
    } else {
      console.log(`   Correct database`);
    }

    // Test 3: Check if users table exists
    console.log('\n3. Checking users table...');
    const tableResult = await sql`
      SELECT table_name, table_schema 
      FROM information_schema.tables 
      WHERE table_name = 'users'
    `;
    
    if (tableResult.length > 0) {
      console.log(`   Users table exists in schema: ${tableResult[0].table_schema}`);
      
      if (tableResult[0].table_schema !== 'public') {
        console.log(`   Warning: Table is in "${tableResult[0].table_schema}" schema, expected "public"`);
      }
    } else {
      console.log('   Users table not found');
      console.log('   Run: pnpm db:migrate');
    }

    // Test 4: Check table columns
    console.log('\n4. Checking table structure...');
    const columnsResult = await sql`
      SELECT column_name, data_type, is_nullable, column_default
      FROM information_schema.columns 
      WHERE table_name = 'users' AND table_schema = 'public'
      ORDER BY ordinal_position
    `;
    
    if (columnsResult.length > 0) {
      console.log('   Table columns:');
      columnsResult.forEach(col => {
        console.log(`      - ${col.column_name} (${col.data_type}, ${col.is_nullable === 'YES' ? 'nullable' : 'not null'})`);
      });
    }

    // Test 5: Check indexes
    console.log('\n5. Checking indexes...');
    const indexesResult = await sql`
      SELECT indexname, indexdef
      FROM pg_indexes 
      WHERE tablename = 'users' AND schemaname = 'public'
    `;
    
    if (indexesResult.length > 0) {
      console.log(`   Found ${indexesResult.length} indexes:`);
      indexesResult.forEach(idx => {
        console.log(`      - ${idx.indexname}`);
      });
    } else {
      console.log('   No indexes found');
    }

    // Test 6: Test basic operations
    console.log('\n6. Testing basic operations...');
    try {
      // Try to insert a test user
      const testId = `test-${Date.now()}`;
      await sql`
        INSERT INTO users (id, email, display_name, created_at, updated_at)
        VALUES (${testId}, ${testId}@test.com, 'Test User', NOW(), NOW())
      `;
      console.log('   INSERT successful');

      // Try to select it
      const selectResult = await sql`
        SELECT id, email FROM users WHERE id = ${testId}
      `;
      console.log('   SELECT successful');

      // Clean up
      await sql`DELETE FROM users WHERE id = ${testId}`;
      console.log('   DELETE successful');
      
    } catch (error) {
      console.log(`   Basic operations failed: ${error.message}`);
    }

    console.log('\nMigration setup test completed!\n');

  } catch (error) {
    console.error('\nTest failed:', error.message);
    console.error('\nMake sure to:');
    console.error('   1. Run the migration: pnpm db:migrate');
    console.error('   2. Check your DATABASE_URL');
    console.error('   3. Ensure PostgreSQL is running\n');
    process.exit(1);
  } finally {
    if (sql) {
      await sql.end();
    }
  }
};

testMigration();

