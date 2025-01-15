# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/crazyscot/engineering_repr/compare/v0.3.1...v1.0.0)

### ⛰️ Features

- Add strict precision option for string conversions; make default sloppy - ([954f10e](https://github.com/crazyscot/engineering_repr/commit/954f10ed73b3a375065e07c6e00fc387b3897287))
- Conversion to f64 and f32 - ([066120a](https://github.com/crazyscot/engineering_repr/commit/066120a010080a0a1af350415e43c88307928777))
- Support the smaller multipliers (milli, micro, etc) - ([17d3b0d](https://github.com/crazyscot/engineering_repr/commit/17d3b0d263c1f16a7e9c5fe0d1bc8ba10bee9216))

### 🧪 Testing

- Minor refactor to improve coverage - ([b78c1ae](https://github.com/crazyscot/engineering_repr/commit/b78c1ae519f4915516b42b37887fe57181ad9162))
- Switch from more-asserts to assertables - ([909c833](https://github.com/crazyscot/engineering_repr/commit/909c83308551b6ff897a7d0e6ace10cdb8b28b01))


## [0.3.1](https://github.com/crazyscot/engineering_repr/compare/v0.3.0...v0.3.1)

### 🐛 Bug Fixes

- ToPrimitive copes gracefully with large negative exponents - ([4753d21](https://github.com/crazyscot/engineering_repr/commit/4753d21ab3eba9a2f43059f4cf3d5e77d834d879))
- Don't crash when significand and exponent combine to be too large for the destination type - ([5f72609](https://github.com/crazyscot/engineering_repr/commit/5f7260955708abc0377524d918af77152bc3922d))


## [0.3.0](https://github.com/crazyscot/engineering_repr/compare/v0.2.0...v0.3.0)

### ⛰️ Features

- [**breaking**] Check for overflow on construction, make conversions to integer infallible - ([7400901](https://github.com/crazyscot/engineering_repr/commit/74009012d022778e9284bdb9b98c93c3be790717))

## [0.2.0](https://github.com/crazyscot/engineering_repr/compare/v0.1.0...v0.2.0)

### ⛰️ Features

- [**breaking**] Serde support is now behind a feature flag - ([30700e7](https://github.com/crazyscot/engineering_repr/commit/30700e78e7b1f12b8f4abe42f21688a0bc0101a5))
- Error implements std::error::Error (via thiserror) - ([e40fd48](https://github.com/crazyscot/engineering_repr/commit/e40fd485a784f4a25bec7e32116f2e8d9928ccd8))

### 📚 Documentation

- Improve README - ([4a3c7f9](https://github.com/crazyscot/engineering_repr/commit/4a3c7f91d7cfea1b9b7b179f7ce84f3d2790f7e2))
- Replace bulky png image with an svg - ([f6e7cd5](https://github.com/crazyscot/engineering_repr/commit/f6e7cd56a4c5c85ddbdc899cfd491a7148e81959))

### 🏗️ Build, packaging & CI

- Run actions on dev branch - ([e81526d](https://github.com/crazyscot/engineering_repr/commit/e81526d1e22b495de7d9501d1e87e19d6af6a991))
- Add dependabot config - ([af773ca](https://github.com/crazyscot/engineering_repr/commit/af773ca7c7007b71ac47fbc378a8935773246374))
- Tidy up dependencies - ([f03f184](https://github.com/crazyscot/engineering_repr/commit/f03f18424e30da7ad20f1c2a4296cd799fe5a4ee))

### ⚙️ Miscellaneous Tasks

- Update workflows and IDE config to suit working with feature flags - ([51f4f21](https://github.com/crazyscot/engineering_repr/commit/51f4f21e0ddad7647f6afa182ec8ba5060bdac90))
- Fix docs badge in README - ([2f9d852](https://github.com/crazyscot/engineering_repr/commit/2f9d8522d66f6f0e0ef369eb917ad24f7de74c89))


## [0.1.0]

### ⛰️ Features

- Precision 0 means automatic; use that when serialising - ([fc1712c](https://github.com/crazyscot/engineering_repr/commit/fc1712c2c84f5e5dcf3521aea8863db4a6180e6f))
- Support serde - ([088ebfd](https://github.com/crazyscot/engineering_repr/commit/088ebfd194a704b31b7f862486f780abf1d50740))
- Initial implementation - ([ee1bcc6](https://github.com/crazyscot/engineering_repr/commit/ee1bcc6fd8136c2b8dbfd25060060ebaa6547c3d))
