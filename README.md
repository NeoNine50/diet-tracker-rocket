Does not work on Windows XP!

Linux users should install sqlite3.

To compile you need Rust with cargo: https://rustup.rs

Install **libsqlite3-dev**

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

