//! Quality Qualitative Coding is an application for doing qualitative coding for interviews between
//! two people where one person is the interviewer and the second the is interviewer who's responses
//! we are interested in coding  

#![warn(
    clippy::all,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]
pub use app::QualityQualitativeCoding;
mod app;
