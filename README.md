# amax-redirect-asi
Rust port of https://github.com/Aib0t/amax-dns-asi

Builds `amax_redirect.asi` as a Blur Plugin: https://github.com/tobii-dev/blur-hooks-rs


```bat
cargo build --release --target i686-pc-windows-msvc
@IF NOT EXIST .\amax\dlls MKDIR .\amax\dlls
COPY /Y ".\target\i686-pc-windows-msvc\release\amax_redirect.dll" ".\amax\dlls\amax_redirect.asi"

@REM COPY /Y ".\amax\dlls\amax_redirect.asi" "<BLUR>\amax\dlls\amax_redirect.asi"
@REM COPY /Y ".\amax\config\amax-redirect.cfg" "<BLUR>\amax\config\amax-redirect.cfg"
```
