import { pgTable, text, timestamp, index } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
  id: text('id').primaryKey(),
  email: text('email').unique(), // Nullable for anonymous users
  display_name: text('display_name'),
  photo_url: text('photo_url'),
  created_at: timestamp('created_at').defaultNow().notNull(),
  updated_at: timestamp('updated_at').defaultNow().notNull(),
}, (table) => ({
  emailIdx: index('users_email_idx').on(table.email),
  createdAtIdx: index('users_created_at_idx').on(table.created_at),
}));

export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert; 