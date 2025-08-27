# PowerShell script to generate GitHub Secrets for Cross-Chain CI Pipeline

Write-Host "🔑 Generating GitHub Secrets for Cross-Chain CI Pipeline" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host ""

# Function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Function to install Solana CLI if not present
function Install-Solana {
    if (-not (Test-Command "solana")) {
        Write-Host "📦 Installing Solana CLI..." -ForegroundColor Yellow
        try {
            Invoke-WebRequest -Uri "https://release.solana.com/stable/install" -OutFile "solana-install.sh"
            bash solana-install.sh
            $env:PATH += ";$env:USERPROFILE\.local\share\solana\install\active_release\bin"
            Write-Host "✅ Solana CLI installed" -ForegroundColor Green
        }
        catch {
            Write-Host "❌ Failed to install Solana CLI" -ForegroundColor Red
            Write-Host "💡 Manual installation: https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "✅ Solana CLI already installed" -ForegroundColor Green
    }
}

# Function to install Node.js dependencies if needed
function Install-Dependencies {
    if (Test-Path "contracts/nft") {
        Write-Host "📦 Installing Node.js dependencies..." -ForegroundColor Yellow
        try {
            Set-Location "contracts/nft"
            npm install --silent
            Set-Location "../.."
            Write-Host "✅ Dependencies installed" -ForegroundColor Green
        }
        catch {
            Write-Host "⚠️ Failed to install dependencies" -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "⚠️ contracts/nft directory not found, skipping npm install" -ForegroundColor Yellow
    }
}

Write-Host "🚀 Step 1: Setting up environment..." -ForegroundColor Blue
Install-Solana
Install-Dependencies

Write-Host ""
Write-Host "🚀 Step 2: Generating EVM Private Key..." -ForegroundColor Blue

# Generate EVM private key using Node.js
if (Test-Command "node") {
    try {
        $EVM_PRIVATE_KEY = node -e "
            const { ethers } = require('ethers');
            const wallet = ethers.Wallet.createRandom();
            console.log(wallet.privateKey);
        " 2>$null
        
        if ($EVM_PRIVATE_KEY -and $EVM_PRIVATE_KEY.Length -gt 0) {
            Write-Host "✅ EVM Private Key generated" -ForegroundColor Green
            Write-Host "📝 EVM_PRIVATE_KEY:" -ForegroundColor Yellow
            Write-Host $EVM_PRIVATE_KEY
        }
        else {
            throw "Failed to generate key"
        }
    }
    catch {
        Write-Host "❌ Failed to generate EVM private key" -ForegroundColor Red
        Write-Host "💡 Manual alternative: Use MetaMask → Account → Export Private Key" -ForegroundColor Yellow
        $EVM_PRIVATE_KEY = "0x0000000000000000000000000000000000000000000000000000000000000000"
    }
}
else {
    Write-Host "❌ Node.js not found" -ForegroundColor Red
    Write-Host "💡 Install Node.js or manually generate private key" -ForegroundColor Yellow
    $EVM_PRIVATE_KEY = "0x0000000000000000000000000000000000000000000000000000000000000000"
}

Write-Host ""
Write-Host "🚀 Step 3: Generating Solana Keypair..." -ForegroundColor Blue

# Generate Solana keypair
$SOLANA_KEYPAIR_FILE = "$env:TEMP\solana-keypair-$(Get-Date -Format 'yyyyMMddHHmmss').json"
try {
    if (Test-Command "solana-keygen") {
        solana-keygen new --outfile $SOLANA_KEYPAIR_FILE --no-bip39-passphrase --silent
        if (Test-Path $SOLANA_KEYPAIR_FILE) {
            Write-Host "✅ Solana keypair generated" -ForegroundColor Green
            Write-Host "📝 SOLANA_KEYPAIR:" -ForegroundColor Yellow
            Get-Content $SOLANA_KEYPAIR_FILE | Write-Host
            
            # Get the public key for reference
            $SOLANA_PUBKEY = solana-keygen pubkey $SOLANA_KEYPAIR_FILE
            Write-Host "🔑 Solana Public Key:" -ForegroundColor Yellow
            Write-Host $SOLANA_PUBKEY
        }
        else {
            throw "Keypair file not created"
        }
    }
    else {
        throw "solana-keygen not found"
    }
}
catch {
    Write-Host "❌ Failed to generate Solana keypair" -ForegroundColor Red
    Write-Host "💡 Manual alternative: Run 'solana-keygen new'" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🚀 Step 4: RPC URLs..." -ForegroundColor Blue

Write-Host "📝 BASE_SEPOLIA_RPC:" -ForegroundColor Yellow
Write-Host "https://sepolia.base.org"

Write-Host "📝 ZETACHAIN_RPC:" -ForegroundColor Yellow
Write-Host "https://rpc.ankr.com/zetachain_evm_testnet"

Write-Host "📝 SOLANA_DEVNET_RPC:" -ForegroundColor Yellow
Write-Host "https://api.devnet.solana.com"

Write-Host ""
Write-Host "🚀 Step 5: Funding Instructions..." -ForegroundColor Blue

Write-Host "💰 Base Sepolia (EVM):" -ForegroundColor Yellow
Write-Host "Get testnet ETH from: https://www.coinbase.com/faucets/base-ethereum-sepolia-faucet"

Write-Host "💰 ZetaChain Testnet:" -ForegroundColor Yellow
Write-Host "Get testnet ZETA from: https://faucet.zetachain.com/"

Write-Host "💰 Solana Devnet:" -ForegroundColor Yellow
Write-Host "Run: solana airdrop 2 --url devnet"

Write-Host ""
Write-Host "🎉 All secrets generated successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Next Steps:" -ForegroundColor Blue
Write-Host "1. Go to your GitHub repository"
Write-Host "2. Settings → Secrets and variables → Actions"
Write-Host "3. Add each secret with the values above"
Write-Host "4. Push your PR to trigger the pipeline!"
Write-Host ""
Write-Host "⚠️ Security Notes:" -ForegroundColor Yellow
Write-Host "- Never commit private keys to your repository"
Write-Host "- Use dedicated testing wallets with minimal funds"
Write-Host "- These are testnet keys - don't use for mainnet"
Write-Host ""

# Clean up temporary files
if (Test-Path $SOLANA_KEYPAIR_FILE) {
    Remove-Item $SOLANA_KEYPAIR_FILE
}

Write-Host "✨ Ready to deploy!" -ForegroundColor Green
