curl -o bitcoin-22.0-osx-signed.dmg https://bitcoincore.org/bin/bitcoin-core-22.0/bitcoin-22.0-osx-signed.dmg
sudo hdiutil attach bitcoin-22.0-osx-signed.dmg
sudo installer -package /Volumes/bitcoin-22.0-osx-signed/bitcoin-22.0-osx-signed.pkg -target /
