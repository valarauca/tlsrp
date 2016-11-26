#tlsrp
Basic Reverse Proxy, HTTP1.1 server, some naive load balancing

Version: 0.0.1 Release Name: Initial

##What is this
A very eary naive attempt at writing a HTTP1.1 server in 100% Rust. This is an early development. It is nowhere near
read for real production. This build _should_ be cross platform. I have not fully test it yet. 

##Where can I learn more?

[It has a website](https://yttrium.rs)

##How do I built it?

###On Fedora 25

```sh
#install build deps
dnf install gcc gcc-c++ openssl-devel
#install rust+rustup+cargo
curl https://sh.rustup.rs -sSf | sh
#add rustup install location to your path
echo 'PATH=$PATH:~/.cargo/bin' >> ~/.bashrc
#restart your shell so rust is installed
git clone https://github.com/valarauca/tlsrp
cd tlsrp
cargo build --release
```

###On CentOS

```sh
#exactly the same as above but change dnf to yum
```

###On Ubuntu/Debian

```sh
#install build deps
apt-get install gcc gcc-c++ libssl-dev curl git
#install rust+rustup+cargo
curl https://sh.rustup.rs -sSf | sh
#add rustup install location to your path
echo 'PATH=$PATH:~/.cargo/bin' >> ~/.bashrc
#restart your shell so rust is installed
git clone https://github.com/valarauca/tlsrp
cd tlsrp
cargo build --release
```

###On Mac

```sh
#install rust+rustup+cargo
curl https://sh.rustup.rs -sSf | sh
#install the free xcode dev tools (llvm/clang/clang++)
xcode-select --install
#install git and openssl
brew install git openssl
#clone the repo
git clone https://github.com/valarauca/tlsrp
cd tlsrp
cargo build --release
```

###On Windows

1. Install visual studio [indirect link](https://www.visualstudio.com/vs/community/) ENSURE YOU GET THE C/C++ TOOLCHAIN not just C#

2. Reboot

3. Install Rust x64 MSVC toolchain [direct link](https://static.rust-lang.org/dist/rust-1.13.0-x86_64-pc-windows-msvc.msi)

4. Ensure Rust executable is reachable via $PATH variable

5. Reboot

6. Install git for windows [direct link](https://github.com/git-for-windows/git/releases/download/v2.10.2.windows.1/Git-2.10.2-64-bit.exe)

7. Reboot

8. Clone the repo `git clone https://github.com/valarauca/tlsrp`

9. `cd tlsrp`

10. `cargo build --release` 

##Q and A

###Q1: What SSL Server is this using?

It is using the native server you have installed. I'm binding too [native-tls](https://crates.io/crates/native-tls)

###Why not use a full Rust TLS implementation like [rustls](https://crates.io/crates/rustls).

rustls is a work in progress, it is still _very_ allocation and cache unfriendly. I have a pull request to help start fixing this. 

###Q3: Where are your integration tests?

I wrote this over the holiday weekend so they're slightly lacking. This was more of an experiment if I _could_ do soemthing more of _if I can do something correctly_. 

###Q4: How many instances of unsafe are there?

1

There are several in unit tests to do unsafe things with enum types to validate properites and functions of those types. 

###Q5: Are there benchmarks?

[Yes](https://yttrium.rs/benchmarks.html)

###Q6: Is there a deployment guide?

If you are okay with systemd [yes](https://yttrium.rs/systemd.html)

###Q7: Is there a technical write up the internal architecture?

[Yes](https://yttrium.rs/whitepaper.html)

#Credits + Dependent libraries

A list of dependent libraries without who this project wouldn't exist. A big thanks to these folks.

* [Steven Fackler of native-tls](https://github.com/sfackler/rust-native-tls)

* [Carl Lerche and Alex Critchton of MIO](https://github.com/carllerche/mio)

* [Marvin Löbel of lazy static](https://github.com/rust-lang-nursery/lazy-static.rs)

* [Kevin K of Clap](https://github.com/kbknapp/clap-rs)

* [Aaron Turon and Alex Critchton of Crossbeam](https://github.com/aturon/crossbeam)

# License:

This software should be considered licensed under the Apache2.0 software license. A full copy of it can be found within
the repository.
