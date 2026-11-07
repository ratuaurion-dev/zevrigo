# Dependensi baru wallet desktop (MOD-15, TSK-M42)

Dibuat otomatis oleh `q113_periksa.py` saat Q113 (DEC-10 bagian 11, DEC-31). Isi: paket yang baru masuk
Cargo.lock lewat `tauri` 2.12.3 dan `tauri-build` 2.7.2, dan semua paket npm di
`apps/wallet/package-lock.json`. Lisensi diperiksa: setiap paket wajib punya minimal satu pilihan lisensi permisif
(MIT, Apache-2.0, BSD, ISC, Zlib, Unicode, CC0, Unlicense, BSL-1.0, MPL-2.0, dan sejenisnya).

## Rust (253 paket baru)

| Lisensi | Jumlah |
|---|---|
| MIT OR Apache-2.0 | 98 |
| MIT | 77 |
| Apache-2.0 OR MIT | 20 |
| Zlib OR Apache-2.0 OR MIT | 16 |
| MIT/Apache-2.0 | 9 |
| Unlicense OR MIT | 7 |
| MPL-2.0 | 5 |
| Apache-2.0/MIT | 3 |
| BSD-3-Clause | 2 |
| BSD-3-Clause OR MIT OR Apache-2.0 | 2 |
| MIT OR Zlib OR Apache-2.0 | 2 |
| Unlicense/MIT | 2 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| Apache-2.0 | 1 |
| Apache-2.0 AND MIT | 1 |
| Apache-2.0 WITH LLVM-exception | 1 |
| BSD-3-Clause AND MIT | 1 |
| BSD-3-Clause/MIT | 1 |
| CC0-1.0 OR MIT-0 OR Apache-2.0 | 1 |
| ISC | 1 |
| MIT OR Apache-2.0 OR Zlib | 1 |
| Zlib | 1 |

| Paket | Versi | Lisensi |
|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| aho-corasick | 1.1.5 | Unlicense OR MIT |
| alloc-no-stdlib | 3.0.0 | BSD-3-Clause |
| alloc-stdlib | 0.3.0 | BSD-3-Clause |
| android_system_properties | 0.1.6 | MIT OR Apache-2.0 |
| atk | 0.18.2 | MIT |
| atk-sys | 0.18.2 | MIT |
| base64 | 0.21.7 | MIT OR Apache-2.0 |
| base64 | 0.23.1 | MIT OR Apache-2.0 |
| bit-set | 0.8.0 | Apache-2.0 OR MIT |
| bit-vec | 0.8.0 | Apache-2.0 OR MIT |
| block2 | 0.6.2 | MIT |
| brotli | 9.1.1 | BSD-3-Clause AND MIT |
| brotli-decompressor | 6.3.0 | BSD-3-Clause/MIT |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| cairo-rs | 0.18.5 | MIT |
| cairo-sys-rs | 0.18.2 | MIT |
| camino | 1.2.6 | MIT OR Apache-2.0 |
| cargo-platform | 0.1.9 | MIT OR Apache-2.0 |
| cargo_metadata | 0.19.2 | MIT |
| cargo_toml | 1.0.1 | Apache-2.0 OR MIT |
| cesu8 | 1.1.0 | Apache-2.0/MIT |
| cfb | 0.14.0 | MIT |
| cfg-expr | 0.15.8 | MIT OR Apache-2.0 |
| chrono | 0.4.45 | MIT OR Apache-2.0 |
| combine | 4.6.9 | MIT |
| cookie | 0.18.2 | MIT OR Apache-2.0 |
| core-foundation | 0.10.1 | MIT OR Apache-2.0 |
| core-graphics | 0.25.0 | MIT OR Apache-2.0 |
| core-graphics-types | 0.2.0 | MIT OR Apache-2.0 |
| crc32fast | 1.5.4 | MIT OR Apache-2.0 |
| crossbeam-channel | 0.5.17 | MIT OR Apache-2.0 |
| cssparser | 0.37.0 | MPL-2.0 |
| cssparser-macros | 0.7.1 | MPL-2.0 |
| ctor | 1.0.13 | Apache-2.0 OR MIT |
| darling | 0.24.1 | MIT |
| darling_core | 0.24.1 | MIT |
| darling_macro | 0.24.1 | MIT |
| dbus | 0.9.12 | Apache-2.0/MIT |
| defmt | 1.2.0 | MIT OR Apache-2.0 |
| defmt-macros | 1.2.0 | MIT OR Apache-2.0 |
| defmt-parser | 1.1.0 | MIT OR Apache-2.0 |
| deranged | 0.5.9 | MIT OR Apache-2.0 |
| derive_more | 2.1.1 | MIT |
| derive_more-impl | 2.1.1 | MIT |
| dirs | 7.0.0 | MIT OR Apache-2.0 |
| dirs-sys | 0.5.0 | MIT OR Apache-2.0 |
| dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT |
| dlopen2 | 0.8.2 | MIT |
| dlopen2_derive | 0.4.3 | MIT |
| dom_query | 0.28.0 | MIT |
| dpi | 0.1.2 | Apache-2.0 AND MIT |
| dtoa | 1.0.11 | MIT OR Apache-2.0 |
| dtoa-short | 0.3.5 | MPL-2.0 |
| dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 |
| embed-resource | 3.0.12 | MIT |
| embed_plist | 1.2.2 | MIT OR Apache-2.0 |
| erased-serde | 0.4.11 | MIT OR Apache-2.0 |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| field-offset | 0.3.6 | MIT OR Apache-2.0 |
| flate2 | 1.1.11 | MIT OR Apache-2.0 |
| foreign-types | 0.5.0 | MIT/Apache-2.0 |
| foreign-types-macros | 0.2.4 | MIT/Apache-2.0 |
| foreign-types-shared | 0.3.1 | MIT/Apache-2.0 |
| gdk | 0.18.2 | MIT |
| gdk-pixbuf | 0.18.5 | MIT |
| gdk-pixbuf-sys | 0.18.0 | MIT |
| gdk-sys | 0.18.2 | MIT |
| gdkwayland-sys | 0.18.2 | MIT |
| gdkx11 | 0.18.2 | MIT |
| gdkx11-sys | 0.18.2 | MIT |
| gio | 0.18.4 | MIT |
| gio-sys | 0.18.1 | MIT |
| glib | 0.18.5 | MIT |
| glib-macros | 0.18.5 | MIT |
| glib-sys | 0.18.1 | MIT |
| glob | 0.3.4 | MIT OR Apache-2.0 |
| gobject-sys | 0.18.0 | MIT |
| gtk | 0.18.2 | MIT |
| gtk-sys | 0.18.2 | MIT |
| gtk3-macros | 0.18.2 | MIT |
| hashbrown | 0.12.3 | MIT OR Apache-2.0 |
| heck | 0.4.1 | MIT OR Apache-2.0 |
| hex | 0.4.3 | MIT OR Apache-2.0 |
| html5ever | 0.39.0 | MIT OR Apache-2.0 |
| iana-time-zone | 0.1.65 | MIT OR Apache-2.0 |
| iana-time-zone-haiku | 0.1.2 | MIT OR Apache-2.0 |
| ico | 0.6.0 | MIT |
| ident_case | 1.0.1 | MIT/Apache-2.0 |
| indexmap | 1.9.3 | Apache-2.0 OR MIT |
| infer | 0.22.0 | MIT |
| javascriptcore-rs | 1.1.2 | MIT |
| javascriptcore-rs-sys | 1.1.1 | MIT |
| jiff | 0.2.39 | Unlicense OR MIT |
| jiff-core | 0.1.2 | Unlicense OR MIT |
| jiff-static | 0.2.39 | Unlicense OR MIT |
| jiff-tzdb | 0.1.9 | Unlicense OR MIT |
| jiff-tzdb-platform | 0.1.3 | Unlicense OR MIT |
| jni | 0.21.1 | MIT/Apache-2.0 |
| jni-sys | 0.3.1 | MIT OR Apache-2.0 |
| jni-sys | 0.4.1 | MIT OR Apache-2.0 |
| jni-sys-macros | 0.4.1 | MIT OR Apache-2.0 |
| json-patch | 4.2.0 | MIT/Apache-2.0 |
| jsonptr | 0.7.1 | MIT OR Apache-2.0 |
| keyboard-types | 0.8.3 | MIT OR Apache-2.0 |
| libappindicator | 0.9.0 | Apache-2.0 OR MIT |
| libappindicator-sys | 0.9.0 | Apache-2.0 OR MIT |
| libdbus-sys | 0.2.7 | Apache-2.0/MIT |
| libloading | 0.7.4 | ISC |
| libredox | 0.1.25 | MIT |
| markup5ever | 0.39.0 | MIT OR Apache-2.0 |
| memoffset | 0.9.1 | MIT |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| muda | 0.20.0 | Apache-2.0 OR MIT |
| ndk | 0.9.0 | MIT OR Apache-2.0 |
| ndk-context | 0.1.1 | MIT OR Apache-2.0 |
| ndk-sys | 0.6.0+11769913 | MIT OR Apache-2.0 |
| new_debug_unreachable | 1.0.6 | MIT |
| num-conv | 0.2.2 | MIT OR Apache-2.0 |
| num_enum | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| num_enum_derive | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| objc2 | 0.6.5 | MIT |
| objc2-app-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-cloud-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-data | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-graphics | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-image | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-location | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-text | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-encode | 4.1.0 | MIT |
| objc2-exception-helper | 0.1.1 | Zlib OR Apache-2.0 OR MIT |
| objc2-foundation | 0.3.2 | MIT |
| objc2-io-surface | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-ui-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-user-notifications | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-web-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| option-ext | 0.2.0 | MPL-2.0 |
| pango | 0.18.3 | MIT |
| pango-sys | 0.18.0 | MIT |
| phf | 0.13.1 | MIT |
| phf_codegen | 0.13.1 | MIT |
| phf_generator | 0.13.1 | MIT |
| phf_macros | 0.13.1 | MIT |
| phf_shared | 0.13.1 | MIT |
| plist | 1.10.3 | MIT |
| png | 0.18.1 | MIT OR Apache-2.0 |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT |
| portable-atomic-util | 0.2.8 | Apache-2.0 OR MIT |
| powerfmt | 0.2.1 | MIT OR Apache-2.0 |
| precomputed-hash | 0.1.1 | MIT |
| proc-macro-crate | 1.3.1 | MIT OR Apache-2.0 |
| proc-macro-crate | 2.0.2 | MIT OR Apache-2.0 |
| proc-macro-error | 1.0.4 | MIT OR Apache-2.0 |
| proc-macro-error-attr | 1.0.4 | MIT OR Apache-2.0 |
| quick-xml | 0.42.0 | MIT |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| redox_users | 0.5.3 | MIT |
| ref-cast | 1.0.27 | MIT OR Apache-2.0 |
| ref-cast-impl | 1.0.27 | MIT OR Apache-2.0 |
| regex | 1.13.1 | MIT OR Apache-2.0 |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 |
| reqwest | 0.13.5 | MIT OR Apache-2.0 |
| same-file | 1.0.6 | Unlicense/MIT |
| schemars | 0.8.22 | MIT |
| schemars | 0.9.0 | MIT |
| schemars | 1.2.3 | MIT |
| schemars_derive | 0.8.22 | MIT |
| selectors | 0.38.0 | MPL-2.0 |
| serde-untagged | 0.1.9 | MIT OR Apache-2.0 |
| serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 |
| serde_repr | 0.1.21 | MIT OR Apache-2.0 |
| serde_spanned | 0.6.9 | MIT OR Apache-2.0 |
| serde_spanned | 1.1.3 | MIT OR Apache-2.0 |
| serde_with | 3.24.0 | MIT OR Apache-2.0 |
| serde_with_macros | 3.24.0 | MIT OR Apache-2.0 |
| serialize-to-javascript | 0.1.2 | MIT OR Apache-2.0 |
| serialize-to-javascript-impl | 0.1.2 | MIT OR Apache-2.0 |
| servo_arc | 0.4.3 | MIT OR Apache-2.0 |
| simd-adler32 | 0.3.10 | MIT |
| siphasher | 1.0.4 | MIT OR Apache-2.0 |
| softbuffer | 0.4.8 | MIT OR Apache-2.0 |
| soup3 | 0.5.0 | MIT |
| soup3-sys | 0.5.0 | MIT |
| string_cache | 0.9.0 | MIT OR Apache-2.0 |
| string_cache_codegen | 0.6.1 | MIT OR Apache-2.0 |
| strsim | 0.11.1 | MIT |
| swift-rs | 1.0.9 | MIT OR Apache-2.0 |
| syn | 1.0.109 | MIT OR Apache-2.0 |
| system-deps | 6.2.2 | MIT OR Apache-2.0 |
| tao | 0.37.2 | Apache-2.0 |
| tao-macros | 0.1.4 | MIT OR Apache-2.0 |
| target-lexicon | 0.12.16 | Apache-2.0 WITH LLVM-exception |
| tauri | 2.12.3 | Apache-2.0 OR MIT |
| tauri-build | 2.7.2 | Apache-2.0 OR MIT |
| tauri-codegen | 2.7.4 | Apache-2.0 OR MIT |
| tauri-macros | 2.7.4 | Apache-2.0 OR MIT |
| tauri-runtime | 2.12.2 | Apache-2.0 OR MIT |
| tauri-runtime-wry | 2.12.3 | Apache-2.0 OR MIT |
| tauri-utils | 2.10.2 | Apache-2.0 OR MIT |
| tauri-winres | 0.3.7 | MIT |
| tendril | 0.5.1 | MIT OR Apache-2.0 |
| time | 0.3.55 | MIT OR Apache-2.0 |
| time-core | 0.1.9 | MIT OR Apache-2.0 |
| time-macros | 0.2.32 | MIT OR Apache-2.0 |
| toml | 0.8.2 | MIT OR Apache-2.0 |
| toml | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_datetime | 0.6.3 | MIT OR Apache-2.0 |
| toml_datetime | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_edit | 0.19.15 | MIT OR Apache-2.0 |
| toml_edit | 0.20.2 | MIT OR Apache-2.0 |
| toml_writer | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 |
| tower-http | 0.6.11 | MIT |
| tray-icon | 0.25.1 | MIT OR Apache-2.0 |
| typeid | 1.0.3 | MIT OR Apache-2.0 |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 |
| urlpattern | 0.6.0 | MIT |
| uuid | 1.29.1 | Apache-2.0 OR MIT |
| version-compare | 0.2.1 | MIT |
| vswhom | 0.1.0 | MIT |
| vswhom-sys | 0.1.3 | MIT |
| walkdir | 2.5.0 | Unlicense/MIT |
| wasm-streams | 0.5.0 | MIT OR Apache-2.0 |
| web_atoms | 0.2.6 | MIT OR Apache-2.0 |
| webkit2gtk | 2.0.2 | MIT |
| webkit2gtk-sys | 2.0.2 | MIT |
| webview2-com | 0.39.1 | MIT |
| webview2-com-macros | 0.8.1 | MIT |
| webview2-com-sys | 0.39.1 | MIT |
| winapi | 0.3.9 | MIT/Apache-2.0 |
| winapi-i686-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 |
| winapi-util | 0.1.11 | Unlicense OR MIT |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 |
| window-vibrancy | 0.8.1 | Apache-2.0 OR MIT |
| windows-sys | 0.45.0 | MIT OR Apache-2.0 |
| windows-targets | 0.42.2 | MIT OR Apache-2.0 |
| windows-version | 0.1.7 | MIT OR Apache-2.0 |
| windows_aarch64_gnullvm | 0.42.2 | MIT OR Apache-2.0 |
| windows_aarch64_msvc | 0.42.2 | MIT OR Apache-2.0 |
| windows_i686_gnu | 0.42.2 | MIT OR Apache-2.0 |
| windows_i686_msvc | 0.42.2 | MIT OR Apache-2.0 |
| windows_x86_64_gnu | 0.42.2 | MIT OR Apache-2.0 |
| windows_x86_64_gnullvm | 0.42.2 | MIT OR Apache-2.0 |
| windows_x86_64_msvc | 0.42.2 | MIT OR Apache-2.0 |
| winnow | 0.5.40 | MIT |
| winreg | 0.56.0 | MIT |
| wry | 0.57.1 | Apache-2.0 OR MIT |
| x11 | 2.21.0 | MIT |
| x11-dl | 2.21.0 | MIT |
| zlib-rs | 0.6.8 | Zlib |

## npm (171 paket, 37 masuk aplikasi)

| Lisensi | Jumlah |
|---|---|
| MIT | 100 |
| MPL-2.0 | 24 |
| Apache-2.0 | 23 |
| Apache-2.0 OR MIT | 13 |
| ISC | 10 |
| BSD-3-Clause | 1 |

| Paket | Versi | Lisensi | Jenis |
|---|---|---|---|
| @jridgewell/gen-mapping | 0.3.13 | MIT | dev |
| @jridgewell/remapping | 2.3.5 | MIT | dev |
| @jridgewell/resolve-uri | 3.1.2 | MIT | dev |
| @jridgewell/sourcemap-codec | 1.6.0 | MIT | dev |
| @jridgewell/trace-mapping | 0.3.31 | MIT | dev |
| @oxc-project/types | 0.156.0 | MIT | dev |
| @rolldown/binding-android-arm-eabi | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-android-arm64 | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-darwin-arm64 | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-darwin-x64 | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-freebsd-x64 | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-linux-arm-gnueabihf | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-linux-arm64-gnu | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-linux-arm64-musl | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-linux-ppc64-gnu | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-linux-s390x-gnu | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-linux-x64-gnu | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-linux-x64-musl | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-openharmony-arm64 | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-win32-arm64-msvc | 1.2.16 | MIT | dev, opsional |
| @rolldown/binding-win32-x64-msvc | 1.2.16 | MIT | dev, opsional |
| @rolldown/pluginutils | 1.0.1 | MIT | dev |
| @tailwindcss/node | 4.3.3 | MIT | dev |
| @tailwindcss/oxide | 4.3.3 | MIT | dev |
| @tailwindcss/oxide-android-arm64 | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-darwin-arm64 | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-darwin-x64 | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-freebsd-x64 | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-linux-arm-gnueabihf | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-linux-arm64-gnu | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-linux-arm64-musl | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-linux-x64-gnu | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-linux-x64-musl | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-wasm32-wasi | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-win32-arm64-msvc | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/oxide-win32-x64-msvc | 4.3.3 | MIT | dev, opsional |
| @tailwindcss/vite | 4.3.3 | MIT | dev |
| @tauri-apps/api | 2.12.3 | Apache-2.0 OR MIT | app |
| @tauri-apps/cli | 2.12.2 | Apache-2.0 OR MIT | dev |
| @tauri-apps/cli-darwin-arm64 | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-darwin-x64 | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-linux-arm-gnueabihf | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-linux-arm64-gnu | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-linux-arm64-musl | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-linux-riscv64-gnu | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-linux-x64-gnu | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-linux-x64-musl | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-win32-arm64-msvc | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-win32-ia32-msvc | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @tauri-apps/cli-win32-x64-msvc | 2.12.2 | Apache-2.0 OR MIT | dev, opsional |
| @types/chai | 5.2.3 | MIT | dev |
| @types/deep-eql | 4.0.2 | MIT | dev |
| @types/estree | 1.0.10 | MIT | dev |
| @types/node | 26.6.6 | MIT | dev |
| @types/qrcode | 1.5.6 | MIT | dev |
| @types/react | 19.3.0 | MIT | app |
| @types/react-dom | 19.3.0 | MIT | dev |
| @typescript/typescript-aix-ppc64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-darwin-arm64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-darwin-x64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-freebsd-arm64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-freebsd-x64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-linux-arm | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-linux-arm64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-linux-loong64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-linux-mips64el | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-linux-ppc64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-linux-riscv64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-linux-s390x | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-linux-x64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-netbsd-arm64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-netbsd-x64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-openbsd-arm64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-openbsd-x64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-sunos-x64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-win32-arm64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @typescript/typescript-win32-x64 | 7.0.2 | Apache-2.0 | dev, opsional |
| @vitejs/plugin-react | 6.2.1 | MIT | dev |
| @vitest/mocker | 5.0.5 | MIT | dev |
| @vitest/spy | 5.0.5 | MIT | dev |
| ansi-regex | 5.0.1 | MIT | app |
| ansi-styles | 4.3.0 | MIT | app |
| assertion-error | 2.0.1 | MIT | dev |
| camelcase | 5.3.1 | MIT | app |
| chai | 6.3.0 | MIT | dev |
| cliui | 6.0.0 | ISC | app |
| color-convert | 2.0.1 | MIT | app |
| color-name | 1.1.4 | MIT | app |
| csstype | 3.2.3 | MIT | app |
| decamelize | 1.2.0 | MIT | app |
| detect-libc | 2.1.2 | Apache-2.0 | dev |
| dijkstrajs | 1.0.3 | MIT | app |
| emoji-regex | 8.0.0 | MIT | app |
| enhanced-resolve | 5.26.1 | MIT | dev |
| es-module-lexer | 2.3.2 | MIT | dev |
| estree-walker | 3.0.3 | MIT | dev |
| expect-type | 1.4.0 | Apache-2.0 | dev |
| fdir | 6.5.0 | MIT | dev |
| find-up | 4.1.0 | MIT | app |
| fsevents | 2.3.3 | MIT | dev, opsional |
| get-caller-file | 2.0.5 | ISC | app |
| graceful-fs | 4.2.11 | ISC | dev |
| is-fullwidth-code-point | 3.0.0 | MIT | app |
| jiti | 2.7.0 | MIT | dev |
| lightningcss | 1.32.0 | MPL-2.0 | dev |
| lightningcss | 1.33.0 | MPL-2.0 | dev |
| lightningcss-android-arm64 | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-android-arm64 | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-darwin-arm64 | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-darwin-arm64 | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-darwin-x64 | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-darwin-x64 | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-freebsd-x64 | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-freebsd-x64 | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-arm-gnueabihf | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-arm-gnueabihf | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-arm64-gnu | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-arm64-gnu | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-arm64-musl | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-arm64-musl | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-x64-gnu | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-x64-gnu | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-x64-musl | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-linux-x64-musl | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-win32-arm64-msvc | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-win32-arm64-msvc | 1.33.0 | MPL-2.0 | dev, opsional |
| lightningcss-win32-x64-msvc | 1.32.0 | MPL-2.0 | dev, opsional |
| lightningcss-win32-x64-msvc | 1.33.0 | MPL-2.0 | dev, opsional |
| locate-path | 5.0.0 | MIT | app |
| lucide-react | 1.66.0 | ISC | app |
| magic-string | 0.30.21 | MIT | dev |
| magic-string | 1.4.5 | MIT | dev |
| magic-string | 1.4.5 | MIT | dev |
| nanoid | 3.3.20 | MIT | dev |
| obug | 3.0.0 | MIT | dev |
| p-limit | 2.3.0 | MIT | app |
| p-locate | 4.1.0 | MIT | app |
| p-try | 2.2.0 | MIT | app |
| path-exists | 4.0.0 | MIT | app |
| picocolors | 1.1.1 | ISC | dev |
| picomatch | 4.0.7 | MIT | dev |
| pngjs | 5.0.0 | MIT | app |
| postcss | 8.5.32 | MIT | dev |
| qrcode | 1.5.4 | MIT | app |
| react | 19.3.1 | MIT | app |
| react-dom | 19.3.1 | MIT | app |
| require-directory | 2.1.1 | MIT | app |
| require-main-filename | 2.0.0 | ISC | app |
| rolldown | 1.2.16 | MIT | dev |
| scheduler | 0.28.0 | MIT | app |
| set-blocking | 2.0.0 | ISC | app |
| source-map-js | 1.2.2 | BSD-3-Clause | dev |
| std-env | 4.3.0 | MIT | dev |
| string-width | 4.2.3 | MIT | app |
| strip-ansi | 6.0.1 | MIT | app |
| tailwindcss | 4.3.3 | MIT | dev |
| tapable | 2.3.3 | MIT | dev |
| tinybench | 6.2.3 | MIT | dev |
| tinyexec | 1.3.1 | MIT | dev |
| tinyglobby | 0.2.17 | MIT | dev |
| typescript | 7.0.2 | Apache-2.0 | dev |
| undici-types | 8.9.0 | MIT | dev |
| vite | 8.3.6 | MIT | dev |
| vitest | 5.0.5 | MIT | dev |
| which-module | 2.0.1 | ISC | app |
| why-is-node-running | 3.2.1 | MIT | dev |
| wrap-ansi | 6.2.0 | MIT | app |
| y18n | 4.0.3 | ISC | app |
| yargs | 15.4.1 | MIT | app |
| yargs-parser | 18.1.3 | ISC | app |
| zustand | 5.1.0 | MIT | app |
