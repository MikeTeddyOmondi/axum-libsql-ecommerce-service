//! Provides SQLX integration for the test database.
//!
//! The database is assumed to be in-memory, and rebuilt from
//! scratch on each start-up.

use anyhow::{Error, Ok, Result};
use axum::http::StatusCode;
use libsql::{params, Connection, Error as LibsqlError};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
// use sqlx::{FromRow, Row, SqlitePool};
use tokio::sync::{Mutex, RwLock};

/// Represents a book, taken from the books table in SQLite.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    /// The post's primary key ID
    pub id: Option<i32>,
    /// The post's title
    pub title: String,
    /// The post's content
    pub content: String,
    /// The post's author (surname, lastname - not enforced)
    pub author_id: String,
    /// The post's createdAt time
    pub created_at: Option<String>,
}

struct PostCache {
    all_books: RwLock<Option<Vec<Post>>>,
}

impl PostCache {
    fn new() -> Self {
        Self {
            all_books: RwLock::new(None),
        }
    }

    async fn all_books(&self) -> Option<Vec<Post>> {
        let lock = self.all_books.read().await;
        lock.clone()
    }

    async fn refresh(&self, books: Vec<Post>) {
        let mut lock = self.all_books.write().await;
        *lock = Some(books);
    }

    async fn invalidate(&self) {
        let mut lock = self.all_books.write().await;
        *lock = None;
    }
}

static CACHE: Lazy<PostCache> = Lazy::new(PostCache::new);

/// Create a database connection pool. Run any migrations.
///
/// ## Returns
/// * A ready-to-use connection pool.
pub async fn init_db() -> Result<Connection> {
    let url = std::env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL must be set");
    let token = std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default();

    let db = libsql::Builder::new_remote(url, token).build().await?;
    let connection = db.connect().unwrap();

    Ok(connection)
}

/// Retrieves all books, sorted by title and then author.
///
/// ## Arguments
/// * `connection_pool` - the connection pool to use.
///
/// ## Returns
/// * A vector of books, or an error.
pub async fn all_books(connection: Connection) -> Result<Vec<Post>> {
    if let Some(all_books) = CACHE.all_books().await {
        Ok(all_books)
    } else {
        // let books = sqlx::query_as::<_, Book>("SELECT * FROM books ORDER BY title,author")
        //     .fetch_all(connection)
        //     .await?;

        let mut results = connection.query("SELECT * FROM users", ()).await.unwrap();

        let mut books: Vec<Post> = Vec::new();

        while let Some(row) = results.next().await.unwrap() {
            let item: Post = Post {
                id: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                content: row.get(2).unwrap(),
                author_id: row.get(3).unwrap(),
                created_at: row.get(4).unwrap(),
            };
            books.push(item);
        }

        CACHE.refresh(books.clone()).await;
        Ok(books)
    }
}

/// Retrieves a single book, by ID
///
/// ## Arguments
/// * `connection_pool` - the database connection pool to use
/// * `id` - the primary key of the book to retrieve
pub async fn book_by_id(connection: Connection, id: i32) -> Result<Post> {
    // Ok(sqlx::query_as::<_, Post>("SELECT * FROM books WHERE id=$1")
    //     .bind(id)
    //     .fetch_one(connection_pool)
    //     .await?)
    let mut results = connection
        .query("SELECT * FROM users WHERE id == ?", params![id])
        .await
        .map_err(|_err: LibsqlError| return LibsqlError::NullValue)?;

    // let row = results.next().await.unwrap().unwrap();
    let mut posts: Vec<Post> = vec![];

    while let Some(row) = results.next().await? {
        let row = Post {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            author_id: row.get(3)?,
            created_at: row.get(4)?,
        };
        posts.push(row)
    }

    match posts.get(0) {
        Some(post) => Ok(post.clone()),
        None => Err(Error::msg("ID NOT FOUND".to_string())),
    }

    // Ok(post)
}

// /// Adds a book to the database.
// ///
// /// ## Arguments
// /// * `connection_pool` - the database connection to use
// /// * `title` - the title of the book to add
// /// * `author` - the author of the book to add
// ///
// /// ## Returns
// /// * The primary key value of the new book
// pub async fn add_book<S: ToString>(connection: Connection, title: S, author_id: S) -> Result<i32> {
//     let title = title.to_string();
//     let author_id = author_id.to_string();

//     let post = Post {
//         id: None,
//         title,
//         content: String::from("foo"),
//         author_id,
//         created_at: None,
//     };

//     // let id = sqlx::query("INSERT INTO books (title, author) VALUES ($1, $2) RETURNING id")
//     //     .bind(title)
//     //     .bind(author)
//     //     .fetch_one(connection_pool)
//     //     .await?
//     //     .get(0);

//     let results = connection
//         .query(
//             "INSERT into posts values (?1, ?2, ?3)",
//             params![post.title.clone(), post.content.clone()],
//         )
//         .await;
//     CACHE.invalidate().await;
//     Ok(id)
// }

// /// Update a book
// ///
// /// ## Arguments
// /// * `connection_pool` - the database connection to use
// /// * `book` - the book object to update. The primary key will be used to
// ///            determine which row is updated.
// pub async fn update_book(connection: Connection, post: &Post) -> Result<()> {
//     // sqlx::query("UPDATE books SET title=$1, author=$2 WHERE id=$3")
//     //     .bind(&book.title)
//     //     .bind(&book.author_id)
//     //     .bind(&book.id)
//     //     .execute(connection_pool)
//     //     .await?;
//     CACHE.invalidate().await;
//     Ok(())
// }

// /// Delete a book
// ///
// /// ## Arguments
// /// * `connection_pool` - the database connection to use
// /// * `id` - the primary key of the book to delete
// pub async fn delete_book(connection_pool: Connection, id: i32) -> Result<()> {
//     sqlx::query("DELETE FROM books WHERE id=$1")
//         .bind(id)
//         .execute(connection_pool)
//         .await?;
//     CACHE.invalidate().await;
//     Ok(())
// }

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[sqlx::test]
//     async fn get_all() {
//         dotenv::dotenv().ok();
//         let cnn = init_db().await.unwrap();
//         let all_rows = all_books(&cnn).await.unwrap();
//         assert!(!all_rows.is_empty());
//     }

//     #[sqlx::test]
//     async fn get_one() {
//         dotenv::dotenv().ok();
//         let cnn = init_db().await.unwrap();
//         let book = book_by_id(&cnn, 1).await.unwrap();
//         assert_eq!(1, book.id);
//         assert_eq!("Hands-on Rust", book.title);
//         assert_eq!("Wolverson, Herbert", book.author);
//     }

//     #[sqlx::test]
//     async fn test_create() {
//         dotenv::dotenv().ok();
//         let cnn = init_db().await.unwrap();
//         let new_id = add_book(&cnn, "Test Book", "Test Author").await.unwrap();
//         let new_book = book_by_id(&cnn, new_id).await.unwrap();
//         assert_eq!(new_id, new_book.id);
//         assert_eq!("Test Book", new_book.title);
//         assert_eq!("Test Author", new_book.author);
//     }

//     #[sqlx::test]
//     async fn test_update() {
//         dotenv::dotenv().ok();
//         let cnn = init_db().await.unwrap();
//         let mut book = book_by_id(&cnn, 2).await.unwrap();
//         book.title = "Updated Book".to_string();
//         update_book(&cnn, &book).await.unwrap();
//         let updated_book = book_by_id(&cnn, 2).await.unwrap();
//         assert_eq!("Updated Book", updated_book.title);
//     }

//     #[sqlx::test]
//     async fn test_delete() {
//         dotenv::dotenv().ok();
//         let cnn = init_db().await.unwrap();
//         let new_id = add_book(&cnn, "DeleteMe", "Test Author").await.unwrap();
//         let _new_book = book_by_id(&cnn, new_id).await.unwrap();
//         delete_book(&cnn, new_id).await.unwrap();
//         let all_books = all_books(&cnn).await.unwrap();
//         assert!(all_books.iter().find(|b| b.title == "DeleteMe").is_none());
//     }
// }
