# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

* Brought the exposed tag constants up to date with RPM 6.0
* Auto-detect available tag constants at build time for compatibility with older librpm versions
* Fixed undefined behavior in `TagData::char()`: C `char` (1 byte) was
  incorrectly cast to Rust `char` (4 bytes), causing out-of-bounds reads.
  The `Char` variant now holds `u8`.

## 0.1.1 (2018-06-10)

* Update links to new project GitHub page and documentation site

## 0.1.0 (2018-04-23)

* Initial release
