Does not work on Windows XP!

Linux users should install sqlite3.

After extracting, right click anywhere in the extracted diet-tracker-rocket-master folder and select Open in Terminal. After typing this command, hit enter: cargo build --release

To compile you need Rust with cargo: https://rustup.rs

Make sure you have copied the templates folder to the target/release folder of your build for the application to start.

Windows users may need to have Visual Studio Build Tools 2019 installed to be able to compile. Choose at least Community edition: https://visualstudio.microsoft.com/downloads/

Linux users may need to have **libsqlite3-dev** installed.

Windows build requires **sqlite3.dll** in the target/{debug, release}/deps folder.

Cross compiling for Win on Linux
--------------------------------
Put **sqlite3.dll** into target/i686-pc-windows-gnu/{debug,release}/deps.
Also on Debian (and possibly others) do:

    sudo apt-get install mingw-w64-i686-dev gcc-mingw-w64-i686
    sudo ln -s /usr/i686-w64-mingw32/lib/libadvapi32.a /usr/i686-w64-mingw32/lib/libAdvapi32.a

and create **.cargo/config** with rustflags only for i686

    [target.i686-pc-windows-gnu]
    linker = "i686-w64-mingw32-gcc"
    rustflags = "-C panic=abort"


