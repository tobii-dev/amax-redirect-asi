cargo build --release --target i686-pc-windows-msvc
@IF NOT EXIST .\amax\dlls MKDIR .\amax\dlls
COPY /Y ".\target\i686-pc-windows-msvc\release\amax_redirect.dll" ".\amax\dlls\amax_redirect.asi"

@REM COPY /Y ".\amax\dlls\amax_redirect.asi" "<BLUR>\amax\dlls\amax_redirect.asi"
@REM COPY /Y ".\amax\config\amax-redirect.cfg" "<BLUR>\amax\config\amax-redirect.cfg"
