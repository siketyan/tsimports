const { platform, arch } = process;

const PLATFORMS = {
  // win32: {
  //   x64: "@tsimports/cli-win32-x64/tsimports.exe",
  //   arm64: "@tsimports/cli-win32-arm64/tsimports.exe",
  // },
  darwin: {
    x64: "@tsimports/cli-darwin-x64/tsimports",
    arm64: "@tsimports/cli-darwin-arm64/tsimports",
  },
  linux: {
    x64: "@tsimports/cli-linux-x64/tsimports",
    arm64: "@tsimports/cli-linux-arm64/tsimports",
  },
  // "linux-musl": {
  //   x64: "@tsimports/cli-linux-x64-musl/tsimports",
  //   arm64: "@tsimports/cli-linux-arm64-musl/tsimports",
  // },
};

const binName = PLATFORMS?.[platform]?.[arch];

if (binName) {
  let binPath;
  try {
    binPath = require.resolve(binName);
  } catch {
    console.warn(
      `The tsimports CLI postinstall script failed to resolve the binary file "${binName}". Running tsimports from the npm package will probably not work correctly.`,
    );
  }
} else {
  console.warn(
    "The tsimports CLI package doesn't ship with prebuilt binaries for your platform yet. " +
      "You can still use the CLI by cloning the tsimports repo from GitHub, " +
      "and follow the instructions there to build the CLI for your platform.",
  );
}
