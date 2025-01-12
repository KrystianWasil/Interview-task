//! ## Task Description
//!
//! The goal is to develop a backend service for shortening URLs using CQRS
//! (Command Query Responsibility Segregation) and ES (Event Sourcing)
//! approaches. The service should support the following features:
//!
//! ## Functional Requirements
//!
//! ### Creating a short link with a random slug
//!
//! The user sends a long URL, and the service returns a shortened URL with a
//! random slug.
//!
//! ### Creating a short link with a predefined slug
//!
//! The user sends a long URL along with a predefined slug, and the service
//! checks if the slug is unique. If it is unique, the service creates the short
//! link.
//!
//! ### Counting the number of redirects for the link
//!
//! - Every time a user accesses the short link, the click count should
//!   increment.
//! - The click count can be retrieved via an API.
//!
//! ### CQRS+ES Architecture
//!
//! CQRS: Commands (creating links, updating click count) are separated from
//! queries (retrieving link information).
//!
//! Event Sourcing: All state changes (link creation, click count update) must be
//! recorded as events, which can be replayed to reconstruct the system's state.
//!
//! ### Technical Requirements
//!
//! - The service must be built using CQRS and Event Sourcing approaches.
//! - The service must be possible to run in Rust Playground (so no database like
//!   Postgres is allowed)
//! - Public API already written for this task must not be changed (any change to
//!   the public API items must be considered as breaking change).

#![allow(unused_variables, dead_code)]

//crates must have
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use commands::CommandHandler;
use queries::QueryHandler;
//event sourcing event enumerate
#[derive(Debug, PartialEq,Clone)]
pub enum Event {
    LinkCreated {
        slug: Slug,
        url: Url,
    },
    LinkAccessed {
        slug: Slug,
    },
    
    UrlChanged{
        slug: Slug,
        new_url: Url,
    },
}

/// All possible errors of the [`UrlShortenerService`].
#[derive(Debug, PartialEq)]
pub enum ShortenerError {
    /// This error occurs when an invalid [`Url`] is provided for shortening.
    InvalidUrl,

    /// This error occurs when an attempt is made to use a slug (custom alias)
    /// that already exists.
    SlugAlreadyInUse,

    /// This error occurs when the provided [`Slug`] does not map to any existing
    /// short link.
    SlugNotFound,
}

/// A unique string (or alias) that represents the shortened version of the
/// URL.
#[derive(Clone, Debug, PartialEq)]
pub struct Slug(pub String);

/// The original URL that the short link points to.
#[derive(Clone, Debug, PartialEq)]
pub struct Url(pub String);

/// Shortened URL representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ShortLink {
    /// A unique string (or alias) that represents the shortened version of the
    /// URL.
    pub slug: Slug,

    /// The original URL that the short link points to.
    pub url: Url,
}

/// Statistics of the [`ShortLink`].
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// [`ShortLink`] to which this [`Stats`] are related.
    pub link: ShortLink,

    /// Count of redirects of the [`ShortLink`].
    pub redirects: u64,
}

/// Commands for CQRS.
pub mod commands {
    use super::{ShortLink, ShortenerError, Slug, Url};

    /// Trait for command handlers.
    pub trait CommandHandler {
        /// Creates a new short link. It accepts the original url and an
        /// optional [`Slug`]. If a [`Slug`] is not provided, the service will generate
        /// one. Returns the newly created [`ShortLink`].
        ///
        /// ## Errors
        ///
        /// See [`ShortenerError`].
        fn handle_create_short_link(
            &mut self,
            url: Url,
            slug: Option<Slug>,
        ) -> Result<ShortLink, ShortenerError>;

        /// Processes a redirection by [`Slug`], returning the associated
        /// [`ShortLink`] or a [`ShortenerError`].
        fn handle_redirect(
            &mut self,
            slug: Slug,
        ) -> Result<ShortLink, ShortenerError>;
        
        fn handle_change_short_link(
        &mut self,
        slug: Slug,
        new_url: Url
    ) -> Result<ShortLink, ShortenerError>;
    }
}

/// Queries for CQRS
pub mod queries {
    use super::{ShortenerError, Slug, Stats};

    /// Trait for query handlers.
    pub trait QueryHandler {
        /// Returns the [`Stats`] for a specific [`ShortLink`], such as the
        /// number of redirects (clicks).
        ///
        /// [`ShortLink`]: super::ShortLink
        fn get_stats(&self, slug: Slug) -> Result<Stats, ShortenerError>;
    }
}

/// CQRS and Event Sourcing-based service implementation
pub struct UrlShortenerService {
    // TODO: add needed fields
    events: Vec<Event>,
}

impl UrlShortenerService {
    /// Creates a new instance of the service
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
    
    //my functions
    
    //record event
    fn record_event(&mut self, event: Event) {
        self.events.push(event);
    }
    //replay events
    fn replay(&self) -> (Vec<ShortLink>, Vec<Stats>) {
        let mut links = Vec::new();
        let mut stats = Vec::new();
    
        for event in &self.events {
            if let Event::LinkCreated { slug, url } = event {
                let short_link = ShortLink {
                    slug: slug.clone(),
                    url: url.clone(),
                };
                links.push(short_link.clone());
                stats.push(Stats {
                    link: short_link,
                    redirects: 0, 
                });
            }
        
        
            // match event {
            //     Event::LinkCreated { slug, url } => {
            //         links.push(ShortLink {
            //             slug: slug.clone(),
            //             url: url.clone
            //         })
            //     }
            // }
        }
    
        for event in &self.events {
            if let Event::LinkAccessed { slug } = event {
                if let Some(stat) = stats.iter_mut().find(|stat| stat.link.slug == *slug) {
                    stat.redirects += 1; 
                }
            }
        }
        
        for event in &self.events {
            if let Event::UrlChanged {slug, new_url} = event {
                if let Some(link) = links.iter_mut().find(|link| link.slug == *slug) {
                    link.url = new_url.clone();
                }
            }
        }
    
        (links, stats)
    }
}

impl commands::CommandHandler for UrlShortenerService {
    fn handle_create_short_link(
        &mut self,
        url: Url,
        slug: Option<Slug>,
    ) -> Result<ShortLink, ShortenerError> {
        // todo!("Implement the logic for creating a short link")
        if !url.0.starts_with("http") || url.0.is_empty() {
            return Err(ShortenerError::InvalidUrl);
        }
        let slug = slug.unwrap_or_else(|| {
            let random_slug: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(6)
                .map(char::from)
                .collect();
            Slug(random_slug)
        });
        //check if slug is unique
        let (links, _) = self.replay();
        if links.iter().any(|link| link.slug == slug) {
            return Err(ShortenerError::SlugAlreadyInUse);
        }
        //record event
        self.record_event(Event::LinkCreated { slug: slug.clone(), url: url.clone() });

        Ok(ShortLink { slug, url })
    }

    fn handle_redirect(
        &mut self,
        slug: Slug,
    ) -> Result<ShortLink, ShortenerError> {
        //todo!("Implement the logic for redirection and incrementing the click counter")
        let (links, _) = self.replay();
        let link = links.into_iter().find(|link| link.slug == slug).ok_or(ShortenerError::SlugNotFound)?;
        self.record_event(Event::LinkAccessed { slug: slug.clone() });
        Ok(link)
    }
    
    fn handle_change_short_link(
        &mut self,
        slug: Slug,
        new_url: Url
    ) -> Result<ShortLink, ShortenerError> {
        let (links, _) = self.replay();
        let mut link = links.into_iter().find(|link| link.slug == slug).ok_or(ShortenerError::SlugNotFound)?;
        link.url = new_url.clone();
        self.record_event(Event::UrlChanged {slug: slug.clone(), new_url: new_url.clone()});
        Ok(link)
    }
        
}


impl queries::QueryHandler for UrlShortenerService {
    fn get_stats(&self, slug: Slug) -> Result<Stats, ShortenerError> {
        //todo!("Implement the logic for retrieving link statistics")
        let (_, stats) = self.replay();

        let stat = stats.into_iter().find(|stat| stat.link.slug == slug).ok_or(ShortenerError::SlugNotFound)?;

        Ok(stat)
    }
}
//my tests
// #[cfg(test)]

// mod tests {
//     use super::*;

//     #[test]
//     fn test_create_short_link() {
//         let mut service = UrlShortenerService::new();
//         let url = Url("https://example.com".to_string());
//         let slug = Slug("example".to_string());

//         let result = service.handle_create_short_link(url.clone(), Some(slug.clone()));
//         assert_eq!(result, Ok(ShortLink { slug: slug.clone(), url: url.clone() }));

//         let result = service.handle_create_short_link(url.clone(), Some(slug.clone()));
//         assert_eq!(result, Err(ShortenerError::SlugAlreadyInUse));
//     }

//     #[test]
//     fn test_redirect() {
//         let mut service = UrlShortenerService::new();
//         let url = Url("https://example.com".to_string());
//         let slug = Slug("example".to_string());

//         let _ = service.handle_create_short_link(url.clone(), Some(slug.clone()));

//         let result = service.handle_redirect(slug.clone());
//         assert_eq!(result, Ok(ShortLink { slug: slug.clone(), url: url.clone() }));
//     }

//     #[test]
//     fn test_get_stats() {
//         let mut service = UrlShortenerService::new();
//         let url = Url("https://example.com".to_string());
//         let slug = Slug("example".to_string());

//         let _ = service.handle_create_short_link(url.clone(), Some(slug.clone()));
//         let _ = service.handle_redirect(slug.clone());

//         let result = service.get_stats(slug.clone());
//         assert_eq!(result, Ok(Stats { link: ShortLink { slug: slug.clone(), url: url.clone() }, redirects: 1 }));
//     }

//     #[test]
//     fn test_redirect_with_multiple_redirects() {
//         let mut service = UrlShortenerService::new();
//         let url = Url("https://example.com".to_string());
//         let slug = Slug("example".to_string());

//         let _ = service.handle_create_short_link(url.clone(), Some(slug.clone()));
//         let _ = service.handle_redirect(slug.clone());
//         let _ = service.handle_redirect(slug.clone());
//         let _ = service.handle_redirect(slug.clone());

//         let result = service.get_stats(slug.clone());
//         assert_eq!(result, Ok(Stats { link: ShortLink { slug: slug.clone(), url: url.clone() }, redirects: 3 }));
//     }
// }

fn main() {
    // example of usage
    let mut service = UrlShortenerService::new();
    let url = Url("https://example-of/verylong-link.com".to_string());
    let new_url = Url("https://example-of/verylong-link/asasasdsad.com".to_string());
    let slug = Slug("short".to_string());
    println!("{:?}", service.handle_create_short_link(url.clone(), Some(slug.clone())));
    println!("{:?}", service.handle_create_short_link(url.clone(), None));
    let _ = service.handle_redirect(slug.clone());
    //let _ = service.handle_redirect(slug.clone());
    println!("{:?}",service.handle_change_short_link(slug.clone(),new_url.clone()));

    let _ = service.handle_redirect(slug.clone());
    
    println!("{:?}", service.get_stats(slug.clone()));
}
