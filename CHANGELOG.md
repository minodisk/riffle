# Changelog

## [0.1.8](https://github.com/minodisk/riffle/compare/v0.1.7...v0.1.8) (2026-09-20)


### Features

* **app:** add Index::clear to empty the folder index and reclaim the space ([#201](https://github.com/minodisk/riffle/issues/201)) ([ff28ab8](https://github.com/minodisk/riffle/commit/ff28ab89cc6986aca2668907851103c1d2e02953))
* **app:** add the index_size and clear_index commands ([#203](https://github.com/minodisk/riffle/issues/203)) ([91b2bce](https://github.com/minodisk/riffle/commit/91b2bce120e70f1d87cbf3e456d15251b2e2280b))
* **app:** add the Move Rejected to Trash menu item and frontend flow ([#240](https://github.com/minodisk/riffle/issues/240)) ([792aa1a](https://github.com/minodisk/riffle/commit/792aa1a6b3052647455ec6b5a1aa2ab0d5d2fdcf))
* **app:** add the settings window's Cache tab ([#204](https://github.com/minodisk/riffle/issues/204)) ([7137943](https://github.com/minodisk/riffle/commit/71379438d31a04366580992fce14bbafb6cdb960))
* **app:** add the trash_rejected command ([#237](https://github.com/minodisk/riffle/issues/237)) ([0ce13d9](https://github.com/minodisk/riffle/commit/0ce13d944dece437fa3b54f4f5172a406cccf4d6))
* **app:** add the viewer empty-state logic ([#217](https://github.com/minodisk/riffle/issues/217)) ([229896e](https://github.com/minodisk/riffle/commit/229896e807f77449dca1e89eb0a817048697e964))
* **app:** disable Clear Cache while a scan runs and show its progress ([#224](https://github.com/minodisk/riffle/issues/224)) ([c067a1a](https://github.com/minodisk/riffle/commit/c067a1a37a9b5e144cb5b3f0e827517b6f4f94ed))
* **app:** emit scan-state and index-clearing signals for settings ([#219](https://github.com/minodisk/riffle/issues/219)) ([b94c49c](https://github.com/minodisk/riffle/commit/b94c49c56bdba94d2ec2d9abdcfd586fd431aacf))
* **app:** filter the strip by portrait or landscape ([#239](https://github.com/minodisk/riffle/issues/239)) ([04b0d55](https://github.com/minodisk/riffle/commit/04b0d554a2fa5731084b0e334c2a98eb73acae98))
* **app:** rescan the open folder on focus and Reload Folder ([#216](https://github.com/minodisk/riffle/issues/216)) ([ecc50a1](https://github.com/minodisk/riffle/commit/ecc50a122d7b8ac6c2b766b1e66b5eef52a870f7))
* **app:** show a clickable empty state over the viewer ([#221](https://github.com/minodisk/riffle/issues/221)) ([26a9817](https://github.com/minodisk/riffle/commit/26a981775fa96d0611840352cf9e97823abfa109))
* **app:** watch the open folder and rescan on change (step 2) ([#220](https://github.com/minodisk/riffle/issues/220)) ([cc5168a](https://github.com/minodisk/riffle/commit/cc5168adb011b1ed36c0252406b4c78a46bf5323))


### Bug Fixes

* **app:** clear the scan entry when the scan task ends ([#212](https://github.com/minodisk/riffle/issues/212)) ([a39e8cc](https://github.com/minodisk/riffle/commit/a39e8cc48d27e19653e2abf3dd8c5322ea5654ae))
* **app:** drop modifier-only keys when the shortcuts setting is read ([#215](https://github.com/minodisk/riffle/issues/215)) ([21e418d](https://github.com/minodisk/riffle/commit/21e418d122ac34c70b1020a0dbe66889e7fbc282))
* **app:** hide a kept pick while XMP is the sidecar format ([#245](https://github.com/minodisk/riffle/issues/245)) ([a86c3e2](https://github.com/minodisk/riffle/commit/a86c3e2bad6207099f49f324972764eeac443d3d))
* **app:** keep the pick of a dirty row across a sidecar format switch ([#232](https://github.com/minodisk/riffle/issues/232)) ([491e5e6](https://github.com/minodisk/riffle/commit/491e5e6f799f55f2815659657f690fab218aa850))
* **app:** key sidecar read errors by the RAW path ([#211](https://github.com/minodisk/riffle/issues/211)) ([81907f3](https://github.com/minodisk/riffle/commit/81907f302b79733f803192d9c80ed5ea44fad6ac))
* **app:** make the filmstrip cell image box aspect-independent ([#207](https://github.com/minodisk/riffle/issues/207)) ([e95df70](https://github.com/minodisk/riffle/commit/e95df702a07f43f56045b52e89bdf60ecf21ae6c))
* **app:** open the filter menu as a fly-out beside the sidebar ([#238](https://github.com/minodisk/riffle/issues/238)) ([099b8fa](https://github.com/minodisk/riffle/commit/099b8fabd92faec06c9b165103ec288e9f8f115e))
* **app:** right-align the filter toggle in the tools row ([#249](https://github.com/minodisk/riffle/issues/249)) ([1194797](https://github.com/minodisk/riffle/commit/1194797cd6bcc091b9a968e024af8ca104ec2507))
* **core:** never rebind an xmp prefix bound to another namespace ([#213](https://github.com/minodisk/riffle/issues/213)) ([5210d6f](https://github.com/minodisk/riffle/commit/5210d6f71d9cc64d36b5da02099f28cfb04a0b66))

## [0.1.7](https://github.com/minodisk/riffle/compare/v0.1.6...v0.1.7) (2026-09-20)


### Features

* **app:** add File &gt; Open Folder… with keymap-driven accelerators ([#199](https://github.com/minodisk/riffle/issues/199)) ([e595692](https://github.com/minodisk/riffle/commit/e595692f608d989f654fda85113b7d0e28fb6021))
* **app:** add Help &gt; Open Log Folder to the app menu ([#194](https://github.com/minodisk/riffle/issues/194)) ([ed02749](https://github.com/minodisk/riffle/commit/ed02749fbcea6ab8713b1eaf5b149d6937ad6fdf))
* **app:** bundle SF Symbol icons for Settings and Undo menu items ([#193](https://github.com/minodisk/riffle/issues/193)) ([072b095](https://github.com/minodisk/riffle/commit/072b0959f596901198ccaab68313b0b2e100cf11))
* **app:** carry the full JPEG size in the focus_crop header for the zoom placeholder ([#173](https://github.com/minodisk/riffle/issues/173)) ([3a0cfbc](https://github.com/minodisk/riffle/commit/3a0cfbc86c1124b66be91b9a0e906fc7ccda9e34))
* **app:** evict stale folders from the index and VACUUM afterwards ([#176](https://github.com/minodisk/riffle/issues/176)) ([cd87f38](https://github.com/minodisk/riffle/commit/cd87f38c75da4ef15136654a9b79a7be8a4d4604))
* **app:** log the first scan's phases with their counts ([#189](https://github.com/minodisk/riffle/issues/189)) ([3baa581](https://github.com/minodisk/riffle/commit/3baa5812aa41317745b50562c8aa2dc320c5ba38))
* **app:** log the second-open path as open lines ([#192](https://github.com/minodisk/riffle/issues/192)) ([d4fff97](https://github.com/minodisk/riffle/commit/d4fff9743648839327c18c8ef2b30afb08f974fa))
* **app:** show native icons on the macOS menu items that have one ([#190](https://github.com/minodisk/riffle/issues/190)) ([c61c0a9](https://github.com/minodisk/riffle/commit/c61c0a995f25e3bb5a48e69239fa156c6214746c))
* **app:** trim the default shortcuts and unify the label defaults ([#191](https://github.com/minodisk/riffle/issues/191)) ([765ed06](https://github.com/minodisk/riffle/commit/765ed0676aed17e86ae5a0752004997553e80af9))


### Bug Fixes

* **app:** cancel the first-batch scan test deterministically ([#183](https://github.com/minodisk/riffle/issues/183)) ([76a6ea8](https://github.com/minodisk/riffle/commit/76a6ea861960c747cb8fa45029ffc9c9eb03ccc1))
* **app:** ignore lone modifier keys during shortcut capture ([#185](https://github.com/minodisk/riffle/issues/185)) ([46ff37a](https://github.com/minodisk/riffle/commit/46ff37a53c7cde5d0a5ac6ccd7cbec2f155a4e6c))
* **app:** keep shortcut overrides that are inactive under the current sidecar format ([#179](https://github.com/minodisk/riffle/issues/179)) ([69ee8fd](https://github.com/minodisk/riffle/commit/69ee8fd0b84e1a0420800ebf6915d2970f55a034))
* **app:** make the pick shortcut editable like any other action ([#180](https://github.com/minodisk/riffle/issues/180)) ([be8b3a3](https://github.com/minodisk/riffle/commit/be8b3a3c0412de0a268f57eca6adea93907f0133))
* **app:** retry a failed sidecar write with a bounded backoff ([#181](https://github.com/minodisk/riffle/issues/181)) ([1c23475](https://github.com/minodisk/riffle/commit/1c23475d1794c0d588099356e1ad2c1f89ef36b7))
* **app:** show sidecar problems in a sticky, dismissible error area ([#182](https://github.com/minodisk/riffle/issues/182)) ([e74a7ac](https://github.com/minodisk/riffle/commit/e74a7acc43faa11d318c81ba24192ab6756c1573))


### Performance Improvements

* **app:** list the folder once for both RAW files and sidecars in scan_folder ([#175](https://github.com/minodisk/riffle/issues/175)) ([9253c1c](https://github.com/minodisk/riffle/commit/9253c1c3b4564dd266e2ed1b03e4b8916bc6d881))

## [0.1.6](https://github.com/minodisk/riffle/compare/v0.1.5...v0.1.6) (2026-09-19)


### Features

* **app:** add a colour label group to the filter menu ([#149](https://github.com/minodisk/riffle/issues/149)) ([931d42b](https://github.com/minodisk/riffle/commit/931d42be6882b5418ca4b9978c5da6817a4ae244))
* **app:** add a pure strip ordering module ([#141](https://github.com/minodisk/riffle/issues/141)) ([f3b5bcd](https://github.com/minodisk/riffle/commit/f3b5bcd2f5d36c05f3e24a4efe14f7e1cf413a01))
* **app:** add a sort choice to the strip pane ([#148](https://github.com/minodisk/riffle/issues/148)) ([7bee9db](https://github.com/minodisk/riffle/commit/7bee9db670305ccc2b098af354415d2a6f147bf3))
* **app:** add and remove individual shortcut keys per action ([#147](https://github.com/minodisk/riffle/issues/147)) ([50eea6b](https://github.com/minodisk/riffle/commit/50eea6b81052d17ca1dca4792c0cacc69b1e5306))
* **app:** add the Auto-advance toggle to the settings window ([#144](https://github.com/minodisk/riffle/issues/144)) ([5ef967d](https://github.com/minodisk/riffle/commit/5ef967d5ec10b2dc3ac34776a6c0c16c8bd1b4f2))
* **app:** add the autoAdvance setting to the backend ([#139](https://github.com/minodisk/riffle/issues/139)) ([062a0d9](https://github.com/minodisk/riffle/commit/062a0d98ab3425544bd9bdb1cb9666c105be5907))
* **app:** advance to the next file after a judgement when Auto-advance is on ([#150](https://github.com/minodisk/riffle/issues/150)) ([546dbb6](https://github.com/minodisk/riffle/commit/546dbb6ddf9f40cdb55a9db3f377d8b83eece982))
* **app:** bind shortcuts with any modifier combination ([#166](https://github.com/minodisk/riffle/issues/166)) ([70580ca](https://github.com/minodisk/riffle/commit/70580ca4d7df691f8b188365f40831fdf10c9109))
* **app:** install updates silently and add Check for Updates… ([#143](https://github.com/minodisk/riffle/issues/143)) ([d978438](https://github.com/minodisk/riffle/commit/d978438034ed4ffddc2d471fefa20669606da161))
* **app:** persist the strip sort order in the settings store ([#153](https://github.com/minodisk/riffle/issues/153)) ([5b047e3](https://github.com/minodisk/riffle/commit/5b047e3ee0495cb4427d392a83cdc40e412c1f38))
* **app:** read folder entries and thumbnails through a separate index connection ([#163](https://github.com/minodisk/riffle/issues/163)) ([7423537](https://github.com/minodisk/riffle/commit/74235373409a2caba63e3b3022dfb71338d17089))
* **app:** set the Riffle app icon ([#165](https://github.com/minodisk/riffle/issues/165)) ([9bbe44b](https://github.com/minodisk/riffle/commit/9bbe44bf90d5143923b5272c86a4ee86768c55a3))
* **app:** show each strip cell's sharpness relative to its neighbours ([#157](https://github.com/minodisk/riffle/issues/157)) ([b3b5fc7](https://github.com/minodisk/riffle/commit/b3b5fc7bffd9052897ae84ba663294f2827de2d9))
* **app:** show the open folder and file in the window title ([#123](https://github.com/minodisk/riffle/issues/123)) ([a09f12f](https://github.com/minodisk/riffle/commit/a09f12f653a1ff1d5426a7ef212aa78b080ba271))
* **app:** split the settings window into tabs ([#170](https://github.com/minodisk/riffle/issues/170)) ([b7a8af3](https://github.com/minodisk/riffle/commit/b7a8af3488f3cb5abe687e26e2d4fcc565f4840e))
* **app:** store the sharpness score in index schema v7 ([#152](https://github.com/minodisk/riffle/issues/152)) ([bace8de](https://github.com/minodisk/riffle/commit/bace8debdd0129749f0fed99598a934015aafb85))
* **app:** undo judgements with Edit &gt; Undo ([#136](https://github.com/minodisk/riffle/issues/136)) ([e463fd4](https://github.com/minodisk/riffle/commit/e463fd4b081c61c8d67fa4473bab0889cfc7464b))
* **core:** score preview sharpness around the focus point at scan time ([#145](https://github.com/minodisk/riffle/issues/145)) ([a2074df](https://github.com/minodisk/riffle/commit/a2074df8025773c87d2bf5e2d53d7ef762c47fea))


### Bug Fixes

* **app,core:** drop ping, release the crop bitmap and guard decode_rgb panics ([#140](https://github.com/minodisk/riffle/issues/140)) ([6767c18](https://github.com/minodisk/riffle/commit/6767c18d21c914af2e3e833e4f4d4d79ba7d390a))
* **app:** defer the Windows update install to quit ([#171](https://github.com/minodisk/riffle/issues/171)) ([c2bca4b](https://github.com/minodisk/riffle/commit/c2bca4bde58f3a20e673efa890c6710aff68aa44))
* **tools:** keep delete_merged_branches.sh from detaching the current worktree ([#167](https://github.com/minodisk/riffle/issues/167)) ([78903ad](https://github.com/minodisk/riffle/commit/78903adb49c52319276f2640d105334e92e9e6aa))
* **tools:** resolve origin/HEAD and fast-forward local main in git:main ([#146](https://github.com/minodisk/riffle/issues/146)) ([abbe752](https://github.com/minodisk/riffle/commit/abbe752ef8211b3ffa6e4628647211dea1650e95))

## [0.1.5](https://github.com/minodisk/riffle/compare/v0.1.4...v0.1.5) (2026-09-19)


### Features

* **app:** add per-format label shortcuts and Ctrl+Alt keys ([#116](https://github.com/minodisk/riffle/issues/116)) ([980d7a8](https://github.com/minodisk/riffle/commit/980d7a84443bb7379bc204a1ffb0e7b65a8e40df))
* **app:** carry the colour label through the writer and set_rating ([#115](https://github.com/minodisk/riffle/issues/115)) ([1b95a4a](https://github.com/minodisk/riffle/commit/1b95a4a971da0b941311b63156fe5bd9925cd503))
* **app:** move settings into a separate settings window ([#120](https://github.com/minodisk/riffle/issues/120)) ([1662f0e](https://github.com/minodisk/riffle/commit/1662f0efe0c452783e1abc9c3f4a67baaa4687fa))
* **app:** show the colour label and set it from the keymap ([#117](https://github.com/minodisk/riffle/issues/117)) ([8674546](https://github.com/minodisk/riffle/commit/8674546019165f115a8c74aa16f8920cc1ff8f4f))
* **app:** store the colour label beside the rating in the index ([#114](https://github.com/minodisk/riffle/issues/114)) ([f3cfeea](https://github.com/minodisk/riffle/commit/f3cfeea65439efa34697ac0eb377d94cff49b22f))
* **core:** read and patch the .dop ColorLabel ([#113](https://github.com/minodisk/riffle/issues/113)) ([e9b77ec](https://github.com/minodisk/riffle/commit/e9b77ecbcf2c715a4ab8a18ae97490d73a6a319f))
* **core:** read and patch xmp:Label ([#111](https://github.com/minodisk/riffle/issues/111)) ([5834a22](https://github.com/minodisk/riffle/commit/5834a224271195e3b1368f745c41267560fecff3))

## [0.1.4](https://github.com/minodisk/riffle/compare/v0.1.3...v0.1.4) (2026-09-18)


### Features

* **app:** add EXIF groups to the filter menu ([#101](https://github.com/minodisk/riffle/issues/101)) ([30b8062](https://github.com/minodisk/riffle/commit/30b806213d96b052aa192a2abbbae0e2d68e3fbb))
* **app:** add shortcut rebind, reset and persistence commands ([#98](https://github.com/minodisk/riffle/issues/98)) ([0ece6f6](https://github.com/minodisk/riffle/commit/0ece6f68f2143906b4d8083fd7f071480eb1c6c1))
* **app:** add the Keyboard Shortcuts menu item and panel ([#102](https://github.com/minodisk/riffle/issues/102)) ([fb2d5aa](https://github.com/minodisk/riffle/commit/fb2d5aaf12defc9e6e1a94ec5c842f3a1592db0f))
* **app:** store the shooting settings in the index ([#100](https://github.com/minodisk/riffle/issues/100)) ([4d39c36](https://github.com/minodisk/riffle/commit/4d39c363b03e8e04629fe51ceb0b3ede17007df1))

## [0.1.3](https://github.com/minodisk/riffle/compare/v0.1.2...v0.1.3) (2026-09-18)


### Features

* **app:** dispatch culling keys through the resolved keymap ([#96](https://github.com/minodisk/riffle/issues/96)) ([9235a9f](https://github.com/minodisk/riffle/commit/9235a9ffee72b507cfce518f304cf416338b50f8))
* **app:** resolve the culling keymap from defaults and shortcut overrides ([#91](https://github.com/minodisk/riffle/issues/91)) ([c3fd55b](https://github.com/minodisk/riffle/commit/c3fd55b8f8a148c15cc31127ab714c96388b11f1))
* **app:** show update download progress and install errors ([#95](https://github.com/minodisk/riffle/issues/95)) ([8670401](https://github.com/minodisk/riffle/commit/867040145353d8b89c18327728b3182b0d1f2226))

## [0.1.2](https://github.com/minodisk/riffle/compare/v0.1.1...v0.1.2) (2026-09-18)


### Features

* **app:** add a pick and rating filter menu to the strip pane ([#88](https://github.com/minodisk/riffle/issues/88)) ([6d085e8](https://github.com/minodisk/riffle/commit/6d085e8c990241dc273d6f349d0152c042c5c58a))
* **app:** add the pick flag for .dop sidecars ([#84](https://github.com/minodisk/riffle/issues/84)) ([3307979](https://github.com/minodisk/riffle/commit/330797940447782bce9b0acd7066337122456c55))
* **app:** show pick/reject as a single dot in the strip cell ([#90](https://github.com/minodisk/riffle/issues/90)) ([6eaff32](https://github.com/minodisk/riffle/commit/6eaff321e63bdf8353e343b8d63942b0952b5cc3))

## [0.1.1](https://github.com/minodisk/riffle/compare/v0.1.0...v0.1.1) (2026-09-18)


### Features

* **app:** add single-instance, opener, and log plugins ([#75](https://github.com/minodisk/riffle/issues/75)) ([51c91ce](https://github.com/minodisk/riffle/commit/51c91ce6c15ef401ccc1a3d32120cb47a92dfccf))
* **app:** add the sidecar format setting (menu, switch, index reset) ([#83](https://github.com/minodisk/riffle/issues/83)) ([2e3ad90](https://github.com/minodisk/riffle/commit/2e3ad9027dcc32e43780397f4d41ed11e3c964ba))
* **app:** list DNG files next to ARW in app and CLI ([#73](https://github.com/minodisk/riffle/issues/73)) ([65cfc39](https://github.com/minodisk/riffle/commit/65cfc397ba05fd532dadaed6fd8dc4aea3bc7b35))
* **app:** open the folder in DxO PhotoLab from the menu ([#80](https://github.com/minodisk/riffle/issues/80)) ([87ebc4f](https://github.com/minodisk/riffle/commit/87ebc4f2f49a4910ede1b7f23f3f42f4dfbd817b))
* **app:** restore the window position and size across launches ([#74](https://github.com/minodisk/riffle/issues/74)) ([32b66c9](https://github.com/minodisk/riffle/commit/32b66c966252f70d3756c8ed435583e65c61b8ce))
* **app:** show an estimated aperture and the Leica focus distance ([#76](https://github.com/minodisk/riffle/issues/76)) ([0e92e29](https://github.com/minodisk/riffle/commit/0e92e296f35a2ec5557a3eaed95e579771c1b64c))
* **app:** write and reconcile the sidecar format selected in settings ([#78](https://github.com/minodisk/riffle/issues/78)) ([89bb2a4](https://github.com/minodisk/riffle/commit/89bb2a489d7ebe40eb3ed20c8568a8609229ecfc))
* **cli:** bench the 1:1 crop without FocusLocation and record DNG numbers ([#77](https://github.com/minodisk/riffle/issues/77)) ([95b313b](https://github.com/minodisk/riffle/commit/95b313b74da2a944064b70105ecf87f1b3bd5cc6))
* **core:** find the embedded JPEGs of a Leica DNG and gate the Sony MakerNote ([#69](https://github.com/minodisk/riffle/issues/69)) ([698fec9](https://github.com/minodisk/riffle/commit/698fec97622d83125e4fcaf41bc8ec733401be6d))
* **core:** read and patch DxO PhotoLab .dop sidecars ([#72](https://github.com/minodisk/riffle/issues/72)) ([57c50a2](https://github.com/minodisk/riffle/commit/57c50a2c5da13a8ebf476fc11df46a5afb282158))

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
