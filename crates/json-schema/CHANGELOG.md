# jjpwrgem-schema

## 0.2.0 — 2026-11-28

### Added

- [dd2afb3](https://github.com/20jasper/JJPWRGEM/commit/dd2afb3a33a15d7946f9440f16ae118117dfe642) validate `items` as a schema or a tuple of schemas.
- [ad4f32d](https://github.com/20jasper/JJPWRGEM/commit/ad4f32de0da87b34f1531fb9158b715a6f3f9f85) boolean schemas are always valid or invalid per their value
- [b0c3a62](https://github.com/20jasper/JJPWRGEM/commit/b0c3a62c6889859643df258f01e8262c46692737) validate `properties` recursively.

### Fixed

- [4a0ff73](https://github.com/20jasper/JJPWRGEM/commit/4a0ff730c41c18f7fc02722508097e915c9da7a6) normalize numbers to correctly detect integers and equality. E.g. 1.5e1 is an integer and equivalent to 15.

## 0.1.2 — 2026-11-25

Version bump only, to follow other packages.

## 0.1.1 — 2026-11-17

### Fixed

- [d209a12](https://github.com/20jasper/JJPWRGEM/commit/d209a12ee24961da18226ec88b7228b10b4bd9fd) fix broken doc links

