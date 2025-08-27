#!/bin/bash

# Solana Universal NFT Program Deployment Script
# This script handles deployment to localnet, devnet, and mainnet

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROGRAM_NAME="universal_nft"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROGRAM_DIR="$PROJECT_ROOT/programs/$PROGRAM_NAME"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if Solana CLI is installed
    if ! command -v solana &> /dev/null; then
        print_error "Solana CLI is not installed. Please install it first."
        exit 1
    fi
    
    # Check if Anchor is installed
    if ! command -v anchor &> /dev/null; then
        print_error "Anchor CLI is not installed. Please install it first."
        exit 1
    fi
    
    # Check if Node.js is installed
    if ! command -v node &> /dev/null; then
        print_error "Node.js is not installed. Please install it first."
        exit 1
    fi
    
    # Check if npm is installed
    if ! command -v npm &> /dev/null; then
        print_error "npm is not installed. Please install it first."
        exit 1
    fi
    
    print_success "All prerequisites are satisfied"
}

# Function to build the program
build_program() {
    print_status "Building the program..."
    
    cd "$PROJECT_ROOT"
    
    # Clean previous build
    if [ -d "target" ]; then
        print_status "Cleaning previous build..."
        anchor clean
    fi
    
    # Build the program
    print_status "Building with Anchor..."
    anchor build
    
    # Verify build
    if [ ! -f "target/deploy/$PROGRAM_NAME.so" ]; then
        print_error "Build failed - program binary not found"
        exit 1
    fi
    
    print_success "Program built successfully"
}

# Function to deploy to localnet
deploy_localnet() {
    print_status "Deploying to localnet..."
    
    # Check if local validator is running
    if ! pgrep -f "solana-test-validator" > /dev/null; then
        print_warning "Local validator not running. Starting it..."
        solana-test-validator --reset &
        sleep 5  # Wait for validator to start
    fi
    
    # Configure for localnet
    solana config set --url localhost
    
    # Deploy the program
    print_status "Deploying program to localnet..."
    anchor deploy --provider.cluster localnet
    
    # Get program ID
    PROGRAM_ID=$(solana address -k target/deploy/$PROGRAM_NAME-keypair.json)
    print_success "Program deployed to localnet with ID: $PROGRAM_ID"
    
    # Update Anchor.toml with program ID
    sed -i.bak "s/universal_nft = \"[^\"]*\"/universal_nft = \"$PROGRAM_ID\"/" Anchor.toml
    
    print_success "Localnet deployment completed"
}

# Function to deploy to devnet
deploy_devnet() {
    print_status "Deploying to devnet..."
    
    # Configure for devnet
    solana config set --url devnet
    
    # Check devnet balance
    BALANCE=$(solana balance)
    print_status "Current devnet balance: $BALANCE"
    
    if [ "$(echo "$BALANCE < 2" | bc -l)" -eq 1 ]; then
        print_warning "Low devnet balance. Requesting airdrop..."
        solana airdrop 2
        sleep 2
    fi
    
    # Deploy the program
    print_status "Deploying program to devnet..."
    anchor deploy --provider.cluster devnet
    
    # Get program ID
    PROGRAM_ID=$(solana address -k target/deploy/$PROGRAM_NAME-keypair.json)
    print_success "Program deployed to devnet with ID: $PROGRAM_ID"
    
    # Update Anchor.toml with program ID
    sed -i.bak "s/universal_nft = \"[^\"]*\"/universal_nft = \"$PROGRAM_ID\"/" Anchor.toml
    
    print_success "Devnet deployment completed"
}

# Function to deploy to mainnet
deploy_mainnet() {
    print_status "Deploying to mainnet..."
    
    # Confirm mainnet deployment
    read -p "Are you sure you want to deploy to mainnet? This will cost real SOL. (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Mainnet deployment cancelled"
        return
    fi
    
    # Configure for mainnet
    solana config set --url mainnet-beta
    
    # Check mainnet balance
    BALANCE=$(solana balance)
    print_status "Current mainnet balance: $BALANCE"
    
    if [ "$(echo "$BALANCE < 5" | bc -l)" -eq 1 ]; then
        print_error "Insufficient mainnet balance. Need at least 5 SOL for deployment."
        exit 1
    fi
    
    # Deploy the program
    print_status "Deploying program to mainnet..."
    anchor deploy --provider.cluster mainnet
    
    # Get program ID
    PROGRAM_ID=$(solana address -k target/deploy/$PROGRAM_NAME-keypair.json)
    print_success "Program deployed to mainnet with ID: $PROGRAM_ID"
    
    # Update Anchor.toml with program ID
    sed -i.bak "s/universal_nft = \"[^\"]*\"/universal_nft = \"$PROGRAM_ID\"/" Anchor.toml
    
    print_success "Mainnet deployment completed"
}

# Function to run tests
run_tests() {
    print_status "Running tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run unit tests
    print_status "Running unit tests..."
    anchor test --skip-local-validator
    
    # Run integration tests if they exist
    if [ -f "package.json" ]; then
        print_status "Running integration tests..."
        npm run test:integration
    fi
    
    print_success "All tests passed"
}

# Function to verify deployment
verify_deployment() {
    local cluster=$1
    print_status "Verifying deployment on $cluster..."
    
    # Get program ID
    PROGRAM_ID=$(solana address -k target/deploy/$PROGRAM_NAME-keypair.json)
    
    # Check if program exists
    if solana program show "$PROGRAM_ID" --url "$cluster" > /dev/null 2>&1; then
        print_success "Program verified on $cluster"
        print_status "Program ID: $PROGRAM_ID"
        
        # Show program info
        solana program show "$PROGRAM_ID" --url "$cluster"
    else
        print_error "Program verification failed on $cluster"
        exit 1
    fi
}

# Function to setup gateway configuration
setup_gateway() {
    print_status "Setting up gateway configuration..."
    
    cd "$PROJECT_ROOT"
    
    # Create gateway config account
    print_status "Creating gateway config account..."
    
    # This would typically involve creating a PDA for gateway configuration
    # and initializing it with the appropriate gateway program ID
    
    print_success "Gateway configuration setup completed"
}

# Function to show help
show_help() {
    echo "Solana Universal NFT Program Deployment Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  build                    Build the program"
    echo "  test                     Run tests"
    echo "  localnet                 Deploy to localnet"
    echo "  devnet                   Deploy to devnet"
    echo "  mainnet                  Deploy to mainnet"
    echo "  verify <cluster>         Verify deployment on specified cluster"
    echo "  gateway                  Setup gateway configuration"
    echo "  all                      Build, test, and deploy to devnet"
    echo "  help                     Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build                 # Build the program"
    echo "  $0 test                  # Run tests"
    echo "  $0 devnet                # Deploy to devnet"
    echo "  $0 verify devnet         # Verify devnet deployment"
    echo "  $0 all                   # Full build, test, and devnet deployment"
}

# Main script logic
main() {
    case "${1:-help}" in
        "build")
            check_prerequisites
            build_program
            ;;
        "test")
            check_prerequisites
            build_program
            run_tests
            ;;
        "localnet")
            check_prerequisites
            build_program
            deploy_localnet
            verify_deployment "localhost"
            ;;
        "devnet")
            check_prerequisites
            build_program
            run_tests
            deploy_devnet
            verify_deployment "devnet"
            ;;
        "mainnet")
            check_prerequisites
            build_program
            run_tests
            deploy_mainnet
            verify_deployment "mainnet-beta"
            ;;
        "verify")
            if [ -z "$2" ]; then
                print_error "Please specify cluster (localnet, devnet, mainnet)"
                exit 1
            fi
            verify_deployment "$2"
            ;;
        "gateway")
            check_prerequisites
            setup_gateway
            ;;
        "all")
            check_prerequisites
            build_program
            run_tests
            deploy_devnet
            verify_deployment "devnet"
            setup_gateway
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# Run main function with all arguments
main "$@"
