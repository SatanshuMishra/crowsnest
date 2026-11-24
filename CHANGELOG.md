# Changelog

## [1.0.2](https://github.com/SatanshuMishra/crowsnest/compare/v1.0.1...v1.0.2) (2026-11-24)


### Bug Fixes

* **monitor:** retry failed stat loads and keep stats on refresh ([10cc77f](https://github.com/SatanshuMishra/crowsnest/commit/10cc77fde123ef48b4f41859b8717ab8e5a047c1))
* **monitor:** retry failed stat loads and keep stats on refresh ([33a308b](https://github.com/SatanshuMishra/crowsnest/commit/33a308bf4116e1996493ab70a66b28f90c366cbd))

## [1.0.1](https://github.com/SatanshuMishra/crowsnest/compare/v1.0.0...v1.0.1) (2026-11-24)


### Bug Fixes

* meet the Wargaming API terms and relicense under Apache 2.0 ([19a1eb1](https://github.com/SatanshuMishra/crowsnest/commit/19a1eb186c443fb85bdec91231f5d7f54a8d0fa6))
* **settings:** show Wargaming player support and the copyright notice ([4dee708](https://github.com/SatanshuMishra/crowsnest/commit/4dee7088b77298e9d8f18586947450df86352cb4))
* **wg:** tell users what to do when Wargaming blocks the application ID ([6119a92](https://github.com/SatanshuMishra/crowsnest/commit/6119a92d90f2a8d8b6c3dc13ce9c36f39cc7c767))

## 1.0.0 (2026-11-24)


### Features

* scaffold Tauri app shell, domain model, CI and signed release pipeline ([fed09cd](https://github.com/SatanshuMishra/crowsnest/commit/fed09cda43314aa24450cb5433c88d6a39735cb1))
* **arena:** parse tempArenaInfo.json and watch the replays folder ([ed5fff5](https://github.com/SatanshuMishra/crowsnest/commit/ed5fff54e9f86159da2e3f1c0bad8bce5e7fe591))
* **game:** detect game installs and resolve the replays folder ([d4152e7](https://github.com/SatanshuMishra/crowsnest/commit/d4152e762a3f4517bba52cf4c231640abdbfa7a1))
* **monitor:** add team tables, stat tiers, player details and status states ([41de8c0](https://github.com/SatanshuMishra/crowsnest/commit/41de8c00ad1dd1dafe3c9320d6a4fcdbdbdda966))
* **rating:** personal rating formula and cached expected values ([0a67544](https://github.com/SatanshuMishra/crowsnest/commit/0a675447b5ec3e32a060ec6b5e6c9123309f55eb))
* **ui:** add setup, settings and update screens ([e0df0f8](https://github.com/SatanshuMishra/crowsnest/commit/e0df0f8f2b4dd77ff7fcd34c0cacab66aee5b8e3))
* **ui:** add shared button, toggle, segmented control and icon components ([2595db4](https://github.com/SatanshuMishra/crowsnest/commit/2595db4b7a7f4bd7a797eda7fea825b240a8d244))
* **wg:** implement Wargaming API client and ship catalog cache ([e155190](https://github.com/SatanshuMishra/crowsnest/commit/e155190ce1c5c047b8bf266803abf05a1d0e5854))


### Bug Fixes

* **arena:** ignore leftover battle files and unrelated removals ([4c289ae](https://github.com/SatanshuMishra/crowsnest/commit/4c289ae2cb7e7fce5491ccf4aacea88a7096bf4e))
* **arena:** keep watching battles after a game update ([d76f72f](https://github.com/SatanshuMishra/crowsnest/commit/d76f72fc685e9e213e18d3ccb99884f24cb2551d))
* **commands:** report why a chosen folder is not a usable install ([9068a1d](https://github.com/SatanshuMishra/crowsnest/commit/9068a1d2f21e7b9188cac0cdb73d62198e24a5c2))
* **model:** serialize player stats fields in camelCase ([34f5cb9](https://github.com/SatanshuMishra/crowsnest/commit/34f5cb9c364a6ac0126c71aab1f4e993a2e9d444))
* **monitor:** keep battles stable across settings, realm and folder changes ([9e55b02](https://github.com/SatanshuMishra/crowsnest/commit/9e55b02acd6b441b03eaef165f8080ad5fb5c9b8))
* **monitor:** show ship stats as loading in player details ([4794bc9](https://github.com/SatanshuMishra/crowsnest/commit/4794bc9f4d94bb68412011f3077140e33e8f5d71))
* **monitor:** tell failed and pending ship stats apart from no battles ([f155f35](https://github.com/SatanshuMishra/crowsnest/commit/f155f35554f18fb1ec9e2ccac6606f8cfccaef13))
* **security:** cap response sizes and the ship list page count ([1f7290c](https://github.com/SatanshuMishra/crowsnest/commit/1f7290cb84573e511c548acae60681540b379803))
* **security:** limit links the app may open to named hosts ([9466ddc](https://github.com/SatanshuMishra/crowsnest/commit/9466ddcaf30f1ad604762709fdbb8c29ace46718))
* **security:** sign updates with a password-protected key ([5e5df35](https://github.com/SatanshuMishra/crowsnest/commit/5e5df353a73f46d7476d8905afab31b237fa21f2))
* **settings:** announce update state changes, not download progress ([3f5fade](https://github.com/SatanshuMishra/crowsnest/commit/3f5fade2c2c4877ac84f9fcf41027d66139c5fd5))
* **settings:** explain why a chosen folder is not a usable install ([c878ba6](https://github.com/SatanshuMishra/crowsnest/commit/c878ba6276024036d2a22d1b20c678a6680dceb7))
* **state:** keep saved settings in request order ([acfd60d](https://github.com/SatanshuMishra/crowsnest/commit/acfd60d534d272525c2a24d6c5e2f013a89e9090))
* **tiers:** rate each statistic by the value it shows ([a15fcf8](https://github.com/SatanshuMishra/crowsnest/commit/a15fcf8c8181cb96d2dbebf54306f257aa5fe893))
* **ui:** drop experience and frags columns in narrow team tables ([ea9bf9a](https://github.com/SatanshuMishra/crowsnest/commit/ea9bf9a31158522463e521a4fcdd2ec83a15f749))
* **ui:** keep focus when the pressed button goes away ([20e380b](https://github.com/SatanshuMishra/crowsnest/commit/20e380be23e3d874231570a585522c33789bda99))
* **ui:** move focus to the heading when the view or setup step changes ([b0424a5](https://github.com/SatanshuMishra/crowsnest/commit/b0424a57c0fe731d17b96287686d41514e20471a))
* **ui:** show the client version with dots ([db6febc](https://github.com/SatanshuMishra/crowsnest/commit/db6febc447b544a8ab259be3f2dbbfc737c3aed1))
* **wg:** retry once when Wargaming reports a source outage ([fdeab53](https://github.com/SatanshuMishra/crowsnest/commit/fdeab533c4dbb9341455b236370d191f4aa450f2))
