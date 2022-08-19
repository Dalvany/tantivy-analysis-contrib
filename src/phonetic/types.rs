/// Allow to set the maximum length in [PhoneticAlgorithm](super::PhoneticAlgorithm).
///
/// If `None` is provided then the phonetic encoder will choose its default.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MaxCodeLength(pub Option<usize>);

/// If text contains multiple words they all get encode if `true` otherwise
/// only the first word will be encoded.
///
/// If `None` is provided, it will be `true`.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Concat(pub Option<bool>);

/// Allow to set the maximum length in [BeiderMorse](super::PhoneticAlgorithm::BeiderMorse).
///
/// If `None` it will use 20.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MaxPhonemeNumber(pub Option<usize>);

/// This is Daitch-Mokotoff rules. They will be parsed.
/// You can find commons-codec's rules [here](https://github.com/apache/commons-codec/blob/rel/commons-codec-1.15/src/main/resources/org/apache/commons/codec/language/dmrules.txt)
///
/// They can be provided using feature `embedded_dm`.
#[cfg(not(feature = "embedded_dm"))]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct DMRule(pub String);

/// This is Daitch-Mokotoff rules. They will be parsed.
/// You can find commons-codec's rules [here](https://github.com/apache/commons-codec/blob/rel/commons-codec-1.15/src/main/resources/org/apache/commons/codec/language/dmrules.txt)
///
/// If `None` is provided then the embedded rules will be used.
#[cfg(feature = "embedded_dm")]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct DMRule(pub Option<String>);

/// Boolean to apply folding (`true`) in Daitch-Mokotoff.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Folding(pub bool);

/// Boolean to allow (`true`) or disallow (`false`) branching
/// for Daitch-Mokotoff.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Branching(pub bool);

/// This boolean allow to generate alternate code, in double metaphone,
/// if different from primary.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Alternate(pub bool);

/// This boolean indicates if Nysiis algorithm should be strict or not.
///
/// Default to `true`.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Strict(pub Option<bool>);

/// This is the mapping for each latin letter for Soundex and Refined
/// Soundex.
///
/// The default is [DEFAULT_US_ENGLISH_MAPPING_SOUNDEX](super::DEFAULT_US_ENGLISH_MAPPING_SOUNDEX).
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Mapping(pub Option<[char; 26]>);

/// Indicate, for Soundex, if `H` and `W` should be treated as silence.
///
/// Default to `true`.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SpecialHW(pub Option<bool>);
