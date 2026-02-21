#!/bin/bash

# Setup script for snap store credentials
# This script helps generate the base64-encoded credentials for GitHub Actions

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

print_status "Setting up snap store credentials for GitHub Actions..."
print_status "Account: kherld.hussein@gmail.com"

# Check if snapcraft is installed
if ! command -v snapcraft &> /dev/null; then
    print_error "snapcraft is not installed. Please install it first:"
    echo "  sudo snap install snapcraft --classic"
    exit 1
fi

# Check if already logged in
if snapcraft whoami &> /dev/null; then
    CURRENT_USER=$(snapcraft whoami)
    print_warning "Already logged into snap store as: $CURRENT_USER"
    if [[ "$CURRENT_USER" == "kherld.hussein@gmail.com" ]]; then
        print_success "Correct account detected: $CURRENT_USER"
    else
        print_warning "Different account detected. Expected: kherld.hussein@gmail.com"
        read -p "Do you want to continue with the current login? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "Please login with the correct account:"
            echo "  snapcraft login"
            print_status "Use email: kherld.hussein@gmail.com"
            exit 1
        fi
    fi
else
    print_status "Please login to snap store first:"
    echo "  snapcraft login"
    print_status "Use email: kherld.hussein@gmail.com"
    print_status "Then run this script again."
    exit 1
fi

# Create temporary directory for credentials
TEMP_DIR=$(mktemp -d)
CREDENTIALS_FILE="$TEMP_DIR/snap_login.json"

print_status "Exporting snap store credentials..."

# Export credentials
snapcraft export-login "$CREDENTIALS_FILE"

if [[ ! -f "$CREDENTIALS_FILE" ]]; then
    print_error "Failed to export credentials"
    exit 1
fi

# Base64 encode the credentials
print_status "Encoding credentials..."
BASE64_CREDENTIALS=$(base64 -w 0 "$CREDENTIALS_FILE")

# Clean up temporary file
rm -rf "$TEMP_DIR"

print_success "Credentials exported and encoded successfully!"
echo ""
print_status "Add the following as a GitHub repository secret:"
echo ""
echo "Secret Name: SNAP_STORE_LOGIN"
echo "Secret Value: $BASE64_CREDENTIALS"
echo ""
print_status "To add this secret:"
echo "1. Go to your GitHub repository"
echo "2. Click on Settings → Secrets and variables → Actions"
echo "3. Click 'New repository secret'"
echo "4. Name: SNAP_STORE_LOGIN"
echo "5. Value: $BASE64_CREDENTIALS"
echo "6. Click 'Add secret'"
echo ""
print_warning "Keep these credentials secure and do not share them publicly!"
