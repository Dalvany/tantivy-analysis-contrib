# Changelog

## [0.12.3](https://github.com/Dalvany/tantivy-analysis-contrib/compare/v0.12.2...v0.12.3) - 2024-10-15

### Other

- bump dependencies
- update release

## 0.9.0

* Add phonex algorithm
* Rework for Tantivy 0.20
* Remove deprecated trim filter
* Remove some features
* Change how to construct ICU's components

## 0.8.0

* Bump icu crates from 3.0 to 4.0

## 0.7.1

* Fix clippy warning

## 0.7.0

* Update derive builder version and Tantivy
* Remove `StopTokenFilter`.

## 0.6.1

* Update rust-icu version to 3.0

## 0.6.0

* Add edge ngram filter
* Deprecate stop word filter. Use [Tantivy's](https://docs.rs/tantivy/0.18.1/tantivy/tokenizer/struct.StopWordFilter.html) instead
* Remove some  ̀From` impl on PathTokenizer, use builder instead.

## 0.5.0

* Add phonetic algorithms with a `PhoneticTokenFilter`

## 0.4.0

* Add clippy workflow
* Fix clippy warnings
* Add a `StopTokenFilter` to remove stop words ([StopFilter](https://lucene.apache.org/core/9_1_0/analysis/common/org/apache/lucene/analysis/core/StopFilter.html))
* Improve documentation

## 0.3.0

* Fix documentation link (I hope)
* Add a `ReverseTokenFilter` that reverse characters of a token. See
  Lucene's [ReverseStringFilter](https://lucene.apache.org/core/9_1_0/analysis/common/org/apache/lucene/analysis/reverse/ReverseStringFilter.html)
* Add an `ElisionTokenFilter` that removes elisions See
  Lucene's [ElisionFilter](https://lucene.apache.org/core/9_1_0/analysis/common/org/apache/lucene/analysis/util/ElisionFilter.html)
* `PathTokenizer` have now a `reverse` field see it can behave
  like [ReversePathHierarchyTokenizer](https://lucene.apache.org/core/9_1_0/analysis/common/org/apache/lucene/analysis/path/ReversePathHierarchyTokenizer.html)

## 0.2.0

* `LengthTokenFilter`
* `TrimTokenFilter`
* `LimitTokenCountFilter`
* `PathTokenizer`

## 0.1.0

* Add Lucene ICU like components
    * `ICUTokenizer`
    * `ICUNormalizer2TokenFilter`
    * `ICUTransformTokenFilter`