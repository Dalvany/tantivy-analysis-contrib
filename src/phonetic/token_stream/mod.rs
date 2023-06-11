//! Module that contains the [TokenStream] implementation. It's this that
//! do the real job.

pub(crate) use beider_morse::BeiderMorseTokenStream;
pub(crate) use daitch_mokotoff::DaitchMokotoffTokenStream;
pub(crate) use double_metaphone::DoubleMetaphoneTokenStream;
pub(crate) use generic::GenericPhoneticTokenStream;

mod beider_morse;
mod daitch_mokotoff;
mod double_metaphone;
mod generic;
