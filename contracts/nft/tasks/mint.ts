import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import isURL from "validator/lib/isURL";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const { isAddress } = hre.ethers.utils;

  const [signer] = await hre.ethers.getSigners();
  if (signer === undefined) {
    throw new Error(
      `Wallet not found. Please, run "npx hardhat account --save" or set PRIVATE_KEY env variable (for example, in a .env file)`
    );
  }

  if (!isAddress(args.contract)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const supportedProtocols = ["https", "ipfs"];

  const isValidTokenUri = isURL(args.tokenUri, {
    require_protocol: true,
    protocols: supportedProtocols,
  });

  if (!isValidTokenUri) {
    throw new Error(
      `Invalid token URI: ${
        args.tokenUri
      }. Supported protocols are: ${supportedProtocols.join(", ")}.`
    );
  }

  const contract = await hre.ethers.getContractAt(
    args.name as "ZetaChainUniversalNFT" | "EVMUniversalNFT",
    args.contract
  );

  const recipient = args.to || signer.address;

  const tx = await contract.safeMint(recipient, args.tokenUri);
  const receipt = await tx.wait();

  const transferEvent = receipt.logs
    .map((log: any) => {
      try {
        return contract.interface.parseLog(log);
      } catch {
        return null;
      }
    })
    .find((parsedLog: any) => parsedLog?.name === "Transfer");

  if (!transferEvent) {
    throw new Error("Transfer event not found in transaction logs.");
  }

  const tokenId = transferEvent?.args?.tokenId;

  if (!tokenId) {
    throw new Error("Transfer event not found in transaction logs.");
  }

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.contract,
        mintTransactionHash: tx.hash,
        recipient: recipient,
        tokenURI: args.tokenUri,
        tokenId: tokenId.toString(),
      })
    );
  } else {
    console.log(`🚀 Successfully minted NFT.
📜 Contract address: ${args.contract}
👤 Recipient: ${recipient}
🆔 Token ID: ${tokenId.toString()}
🔗 Transaction hash: ${tx.hash}`);
  }
};

export const nftMint = task("nft:mint", "Mint an NFT", main)
  .addParam("contract", "The address of the deployed NFT contract")
  .addOptionalParam(
    "to",
    "The recipient address, defaults to the signer address"
  )
  .addParam("tokenUri", "The metadata URI of the token")
  .addOptionalParam(
    "name",
    "The contract name to interact with",
    "ZetaChainUniversalNFT"
  )
  .addFlag("json", "Output the result in JSON format");
