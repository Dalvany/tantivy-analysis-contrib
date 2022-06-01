# Changelog

## Next

* Add clippy workflow
* Fix clippy warnings
* Add a `StopTokenFilter` to remove stop words ([StopFilter](https://lucene.apache.org/core/9_1_0/analysis/common/org/apache/lucene/analysis/core/StopFilter.html))

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