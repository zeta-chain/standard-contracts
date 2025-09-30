import path from "path";
import fs from "fs";

/**
 * Load contract artifacts (ABI & bytecode) from the current working directory.
 *
 * This function expects compiled artifacts in your project's out/ directory.
 * Run 'forge build' before using commands that call this function.
 *
 * @param contractName - Name of the contract (e.g., "ZetaChainUniversalNFT")
 * @param sourceName - Source file name (defaults to {contractName}.sol)
 * @returns Object containing contract ABI and bytecode
 * @throws Error if artifacts are not found or cannot be loaded
 *
 * @example
 * const { abi, bytecode } = loadContractArtifacts("ZetaChainUniversalNFT");
 */
export const loadContractArtifacts = (
  contractName: string,
  sourceName?: string
) => {
  const sourcePath = sourceName || `${contractName}.sol`;
  const artifactPath = path.join(
    process.cwd(),
    `out/${sourcePath}/${contractName}.json`
  );

  // Check if the artifact file exists before attempting to read
  if (!fs.existsSync(artifactPath)) {
    const outDir = path.join(process.cwd(), "out");
    const hasOutDir = fs.existsSync(outDir);

    throw new Error(
      `Contract artifacts not found for ${contractName}\n\n` +
        `Expected location: ${artifactPath}\n` +
        `Current directory: ${process.cwd()}\n\n` +
        (hasOutDir
          ? `The 'out/' directory exists but doesn't contain ${sourcePath}/${contractName}.json\n` +
            `Available contracts in out/:\n${listOutDirectory(outDir)}\n`
          : `The 'out/' directory was not found.\n`) +
        `\nTo fix this issue:\n` +
        `  1. Ensure you're running this command from your project root\n` +
        `  2. Run 'forge build' to compile your contracts\n` +
        `  3. Verify that ${contractName}.sol exists in your contracts/ directory\n`
    );
  }

  try {
    const artifact = JSON.parse(fs.readFileSync(artifactPath, "utf8"));

    // Validate that the artifact has the required fields
    if (!artifact.abi) {
      throw new Error("Artifact file is missing 'abi' field");
    }
    if (!("bytecode" in artifact)) {
      throw new Error("Artifact file is missing 'bytecode' field");
    }

    return {
      abi: artifact.abi,
      bytecode: artifact.bytecode,
    } as { abi: any; bytecode: string };
  } catch (error) {
    if (error instanceof SyntaxError) {
      throw new Error(
        `Failed to parse artifact file at ${artifactPath}\n` +
          `The file may be corrupted. Try running 'forge clean && forge build'`
      );
    }

    const message = error instanceof Error ? error.message : String(error);
    throw new Error(
      `Unable to load contract artifacts for ${contractName}: ${message}`
    );
  }
};

/**
 * Helper function to list contents of out/ directory for debugging.
 * Lists up to 10 contract directories to help users identify what's available.
 */
function listOutDirectory(outDir: string): string {
  try {
    const contents = fs.readdirSync(outDir, { withFileTypes: true });
    const dirs = contents
      .filter((dirent) => dirent.isDirectory())
      .map((dirent) => `  - ${dirent.name}/`)
      .slice(0, 10);

    if (dirs.length === 0) {
      return "  (empty directory)";
    }

    return dirs.join("\n") + (contents.length > 10 ? "\n  ... and more" : "");
  } catch {
    return "  (unable to read directory)";
  }
}