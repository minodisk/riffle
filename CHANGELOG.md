# Changelog

## 0.1.0 (2026-09-18)


### Features

* **app-ui:** add ArrowUp/ArrowDown, WASD and HJKL paging keys ([#14](https://github.com/minodisk/riffle/issues/14)) ([b3787c8](https://github.com/minodisk/riffle/commit/b3787c8a9e5dd070571f418b4c0fb42390a69332))
* **app:** add a virtualised left thumbnail filmstrip ([#19](https://github.com/minodisk/riffle/issues/19)) ([2913a5e](https://github.com/minodisk/riffle/commit/2913a5e0773e7520549c3f7b6f4d52f959f9d22f))
* **app:** add an async focus_crop command returning a raw RGBA crop ([#30](https://github.com/minodisk/riffle/issues/30)) ([f839a21](https://github.com/minodisk/riffle/commit/f839a2158a1d47e5cc2f73e6b81a506d890b9255))
* **app:** add tauri-plugin-updater and an in-app update check ([#50](https://github.com/minodisk/riffle/issues/50)) ([2eadc47](https://github.com/minodisk/riffle/commit/2eadc47987dd8b9cca39bffaa8fa27777bbbf54a))
* **app:** draw the focus box on the preview ([#20](https://github.com/minodisk/riffle/issues/20)) ([808a58a](https://github.com/minodisk/riffle/commit/808a58afc37b4d8a608e44c870a7d8c235c557ce))
* **app:** draw the focus crosshair in green with a dark outline ([#48](https://github.com/minodisk/riffle/issues/48)) ([e450a3e](https://github.com/minodisk/riffle/commit/e450a3e14577b2e9b6cef9c56e9d79c8fdbf0c2d))
* **app:** frontend folder button, canvas, decode worker, arrow-key paging ([#7](https://github.com/minodisk/riffle/issues/7)) ([9d3adf2](https://github.com/minodisk/riffle/commit/9d3adf22dabe7edc22965632696f2204813f98ec))
* **app:** hide the focus box by default and draw it inverted ([#37](https://github.com/minodisk/riffle/issues/37)) ([39c4072](https://github.com/minodisk/riffle/commit/39c4072a22c0e52b5725c63129f2c7d02344e273))
* **app:** index folders in SQLite and scan them in the background ([#18](https://github.com/minodisk/riffle/issues/18)) ([0c5085f](https://github.com/minodisk/riffle/commit/0c5085f3ccfa807c5b4ecf501b1e7f61e29e8478))
* **app:** mark the focus point with a crosshair instead of a box ([#42](https://github.com/minodisk/riffle/issues/42)) ([4663d7c](https://github.com/minodisk/riffle/commit/4663d7c43ad1c84191424a5052fca57666d85b52))
* **app:** open a folder by drag-and-drop ([#21](https://github.com/minodisk/riffle/issues/21)) ([f610d71](https://github.com/minodisk/riffle/commit/f610d71c8610433f3e295786ad0973f489e3474b))
* **app:** rate files from the keyboard with rating badges ([#41](https://github.com/minodisk/riffle/issues/41)) ([6109790](https://github.com/minodisk/riffle/commit/6109790fa20d0104542c61b54b51f9de98b33f5d))
* **app:** reconcile sidecars on folder open, and flush dirty rows ([#36](https://github.com/minodisk/riffle/issues/36)) ([60941e2](https://github.com/minodisk/riffle/commit/60941e2fe95a11d17e807c0311dbb1de321ddfb3))
* **app:** reopen the last opened folder on launch ([#46](https://github.com/minodisk/riffle/issues/46)) ([b510a4b](https://github.com/minodisk/riffle/commit/b510a4bfa6ceb38c101ee6cf5d6e9618a3029ed1))
* **app:** report the read and decode time of a focus crop ([#51](https://github.com/minodisk/riffle/issues/51)) ([325be39](https://github.com/minodisk/riffle/commit/325be39fa57fe044ae6085d7c522f31076b9f504))
* **app:** Rust commands for folder pick, ARW listing, and preview bytes ([#6](https://github.com/minodisk/riffle/issues/6)) ([9e9deae](https://github.com/minodisk/riffle/commit/9e9deaec09c0fd0a5b8397bff5648f0fedbab045))
* **app:** scaffold Tauri 2 app crate and move CI to macOS ([#4](https://github.com/minodisk/riffle/issues/4)) ([1c43d3a](https://github.com/minodisk/riffle/commit/1c43d3ad327815581e85377c5843280c3540b6c6))
* **app:** show the shooting settings in a right metadata pane ([#26](https://github.com/minodisk/riffle/issues/26)) ([e6c33a0](https://github.com/minodisk/riffle/commit/e6c33a0eaba786d33ca0d64dd13dc6897d2e6e5d))
* **app:** store ratings in the index and coalesce XMP sidecar writes ([#33](https://github.com/minodisk/riffle/issues/33)) ([d721a00](https://github.com/minodisk/riffle/commit/d721a003e71f2d0f8b920143654f22bf019019ae))
* **app:** tidy up the rating display across canvas, strip, and meta pane ([#49](https://github.com/minodisk/riffle/issues/49)) ([cf1f3a5](https://github.com/minodisk/riffle/commit/cf1f3a55e7b9cc2f2b9dcc3a1a7664f6eacd992c))
* **app:** toggle a 1:1 focus check with Space ([#32](https://github.com/minodisk/riffle/issues/32)) ([a2decfc](https://github.com/minodisk/riffle/commit/a2decfc020af21f33fafadae4d7ee555997401a1))
* **app:** toggle timing logs from a Debug menu ([#44](https://github.com/minodisk/riffle/issues/44)) ([b821fa3](https://github.com/minodisk/riffle/commit/b821fa3c8bf5e883778abb9f1631d3230b990ea1))
* **core:** add a ranged JpgFromRaw read and focus-point RGBA crop ([#29](https://github.com/minodisk/riffle/issues/29)) ([c2eafea](https://github.com/minodisk/riffle/commit/c2eafeab5166be37c7a66c8537fdaa0b7439a566))
* **core:** parallel extraction with rayon and a CLI scan benchmark ([#17](https://github.com/minodisk/riffle/issues/17)) ([ff7f6f6](https://github.com/minodisk/riffle/commit/ff7f6f6a9ab1229a132e0c37f6a414d0ddd05d7e))
* **core:** read, write and patch XMP sidecar ratings ([#31](https://github.com/minodisk/riffle/issues/31)) ([c540dc7](https://github.com/minodisk/riffle/commit/c540dc7e2ed161c31f36912a18ad15211bcec9d9))


### Bug Fixes

* **app:** debounce the 1:1 crop request while the window resizes ([#60](https://github.com/minodisk/riffle/issues/60)) ([3f80bf0](https://github.com/minodisk/riffle/commit/3f80bf07f92e8bebee24246eb608fd2dc8ca86fd))
* **app:** keep the meta pane layout stable while paging ([#54](https://github.com/minodisk/riffle/issues/54)) ([f1ba56d](https://github.com/minodisk/riffle/commit/f1ba56d0dfee7833f2880f0f15bfefde6599e3b4))
* **app:** stop re-requesting every pending thumbnail on each scan tick ([#56](https://github.com/minodisk/riffle/issues/56)) ([d41b3bf](https://github.com/minodisk/riffle/commit/d41b3bf034738928a6e5e96ff7bd98ac213b1ec3))
* **app:** stop the folder dialog deadlocking the main thread ([#11](https://github.com/minodisk/riffle/issues/11)) ([e5d4fff](https://github.com/minodisk/riffle/commit/e5d4fff498f083bddf275f2ddeff07e2a08db599))
