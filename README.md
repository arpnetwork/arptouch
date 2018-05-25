# arptouch

Multitouch interactive program for Android.

## Android standalone toolchain

```bash
# Download Android NDK
curl -O http://dl.google.com/android/repository/android-ndk-r10e-linux-x86_64.zip
unzip android-ndk-r10e-linux-x86_64.zip

# Make a standalone toolchain
android-ndk-r10e/build/tools/make-standalone-toolchain.sh \
    --platform=android-18 --toolchain=arm-linux-androideabi-clang3.6 \
    --install-dir=/tmp/android-18-toolchain --ndk-dir=android-ndk-r10e/ --arch=arm
```

## Building [`libevdev.a`](https://www.freedesktop.org/wiki/Software/libevdev/) for Android

```bash
# Cross compile for Android
export PATH=/tmp/android-18-toolchain/bin:$PATH
export CC=arm-linux-androideabi-gcc

git clone git://anongit.freedesktop.org/libevdev
cd libevdev
sed -i 's/SUBDIRS = .*/SUBDIRS = libevdev/g' Makefile.am
autoreconf -fvi
./configure --host=arm-linux-androideabi --disable-test-run --prefix=/tmp/libevdev
make && make install
```

## Building

Clone from github:

```bash
git clone https://github.com/arpnetwork/arptouch.git
cd arptouch
```

Copy libevdev.a for Android to `lib` folder:

```bash
cp /tmp/libevdev/lib/libevdev.a lib/
```

Acquire a Rust standard library for Android platform:

```bash
rustup target add arm-linux-androideabi
```

Building with Android standalone toolchain:

```bash
export PATH=/tmp/android-18-toolchain/bin:$PATH
cargo build --release
```
