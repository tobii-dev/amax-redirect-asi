cargo build --release
COPY /Y ".\target\i686-pc-windows-msvc\release\amax_redirect.dll" ".\amax\amax-redirect.asi"
