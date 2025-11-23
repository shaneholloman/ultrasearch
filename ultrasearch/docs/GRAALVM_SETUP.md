# GraalVM setup for `extractous_backend`

Use this when enabling the heavy Extractous content pipeline.

## Download (Windows x64)
- GraalVM JDK CE 23.0.2 (Windows x64) from Oracle:
  - URL: https://download.oracle.com/graalvm/23/archive/graalvm-jdk-23.0.2_windows-x64_bin.zip
  - SHA256: `501da4f5610e64a8644df92773e1aba559d1c542a84aacea9b37d469aa9da8a7`

Verify checksum (PowerShell):
```powershell
Get-FileHash .\graalvm-jdk-23.0.2_windows-x64_bin.zip -Algorithm SHA256
```

## Install
1. Unzip to e.g. `C:\tools\graalvm-23.0.2`.
2. Set env vars (add to user env or your shell profile):
   - `GRAALVM_HOME=C:\tools\graalvm-23.0.2`
   - `JAVA_HOME=%GRAALVM_HOME%`
   - `PATH=%GRAALVM_HOME%\bin;%PATH%`
3. Validate:
   ```powershell
   & $Env:GRAALVM_HOME\bin\java -version
   # should show GraalVM 23.x
   ```

## Build guards
- `crates/content-extractor/build.rs` enforces GraalVM 23.x when the Cargo feature `extractous_backend` is enabled. It will fail the build if `java -version` does not report GraalVM 23.x.
- Keep the feature off for lightweight builds: `cargo build -p content-extractor`.
- Enable when ready: `cargo build -p content-extractor --features extractous_backend`.

## Smoke test
A gated smoke test runs when `extractous_backend` is enabled **and** `GRAALVM_HOME`/`JAVA_HOME` are set. It checks that the Extractous backend advertises support for a simple `.txt` file without invoking the full runtime (see `crates/content-extractor/src/lib.rs`).

