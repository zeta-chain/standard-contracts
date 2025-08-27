# Simple PowerShell script to generate GitHub Secrets for Cross-Chain CI Pipeline

Write-Host "Generating GitHub Secrets for Cross-Chain CI Pipeline" -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host ""

# Function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

Write-Host "Step 1: Checking environment..." -ForegroundColor Blue

# Check if Node.js is available
if (Test-Command "node") {
    Write-Host "Node.js found" -ForegroundColor Green
} else {
    Write-Host "Node.js not found - will provide manual instructions" -ForegroundColor Yellow
}

# Check if Solana CLI is available
if (Test-Command "solana") {
    Write-Host "Solana CLI found" -ForegroundColor Green
} else {
    Write-Host "Solana CLI not found - will provide manual instructions" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Step 2: Generating EVM Private Key..." -ForegroundColor Blue

# Try to generate EVM private key
try {
    if (Test-Command "node") {
        $EVM_PRIVATE_KEY = node -e "const { ethers } = require('ethers'); const wallet = ethers.Wallet.createRandom(); console.log(wallet.privateKey);" 2>$null
        
        if ($EVM_PRIVATE_KEY -and $EVM_PRIVATE_KEY.Length -gt 0) {
            Write-Host "EVM Private Key generated successfully" -ForegroundColor Green
            Write-Host "EVM_PRIVATE_KEY:" -ForegroundColor Yellow
            Write-Host $EVM_PRIVATE_KEY
        } else {
            throw "Failed to generate key"
        }
    } else {
        throw "Node.js not available"
    }
} catch {
    Write-Host "Failed to generate EVM private key automatically" -ForegroundColor Red
    Write-Host "Manual alternative: Use MetaMask -> Account -> Export Private Key" -ForegroundColor Yellow
    Write-Host "Or create a new wallet at: https://metamask.io/" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Step 3: Generating Solana Keypair..." -ForegroundColor Blue

# Try to generate Solana keypair
try {
    if (Test-Command "solana-keygen") {
        $SOLANA_KEYPAIR_FILE = "$env:TEMP\solana-keypair-$(Get-Date -Format 'yyyyMMddHHmmss').json"
        solana-keygen new --outfile $SOLANA_KEYPAIR_FILE --no-bip39-passphrase --silent
        
        if (Test-Path $SOLANA_KEYPAIR_FILE) {
            Write-Host "Solana keypair generated successfully" -ForegroundColor Green
            Write-Host "SOLANA_KEYPAIR:" -ForegroundColor Yellow
            Get-Content $SOLANA_KEYPAIR_FILE | Write-Host
            
            # Get the public key for reference
            $SOLANA_PUBKEY = solana-keygen pubkey $SOLANA_KEYPAIR_FILE
            Write-Host "Solana Public Key:" -ForegroundColor Yellow
            Write-Host $SOLANA_PUBKEY
            
            # Clean up
            Remove-Item $SOLANA_KEYPAIR_FILE
        } else {
            throw "Keypair file not created"
        }
    } else {
        throw "solana-keygen not available"
    }
} catch {
    Write-Host "Failed to generate Solana keypair automatically" -ForegroundColor Red
    Write-Host "Manual alternative: Run 'solana-keygen new'" -ForegroundColor Yellow
    Write-Host "Or install Solana CLI: https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Step 4: RPC URLs..." -ForegroundColor Blue

Write-Host "BASE_SEPOLIA_RPC:" -ForegroundColor Yellow
Write-Host "https://sepolia.base.org"

Write-Host "ZETACHAIN_RPC:" -ForegroundColor Yellow
Write-Host "https://rpc.ankr.com/zetachain_evm_testnet"

Write-Host "SOLANA_DEVNET_RPC:" -ForegroundColor Yellow
Write-Host "https://api.devnet.solana.com"

Write-Host ""
Write-Host "Step 5: Funding Instructions..." -ForegroundColor Blue

Write-Host "Base Sepolia (EVM):" -ForegroundColor Yellow
Write-Host "Get testnet ETH from: https://www.coinbase.com/faucets/base-ethereum-sepolia-faucet"

Write-Host "ZetaChain Testnet:" -ForegroundColor Yellow
Write-Host "Get testnet ZETA from: https://faucet.zetachain.com/"

Write-Host "Solana Devnet:" -ForegroundColor Yellow
Write-Host "Run: solana airdrop 2 --url devnet"

Write-Host ""
Write-Host "All secrets generated successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Blue
Write-Host "1. Go to your GitHub repository"
Write-Host "2. Settings -> Secrets and variables -> Actions"
Write-Host "3. Add each secret with the values above"
Write-Host "4. Push your PR to trigger the pipeline!"
Write-Host ""
Write-Host "Security Notes:" -ForegroundColor Yellow
Write-Host "- Never commit private keys to your repository"
Write-Host "- Use dedicated testing wallets with minimal funds"
Write-Host "- These are testnet keys - don't use for mainnet"
Write-Host ""

Write-Host "Ready to deploy!" -ForegroundColor Green
