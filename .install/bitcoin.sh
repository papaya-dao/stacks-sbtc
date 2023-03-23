if [[ $OSTYPE == 'darwin'* ]]
then

    # curl -o bitcoin-22.0-osx-signed.dmg https://bitcoincore.org/bin/bitcoin-core-22.0/bitcoin-22.0-osx-signed.dmg
    # sudo hdiutil attach bitcoin-22.0-osx-signed.dmg
    # sudo installer -package /Volumes/Bitcoin-Core/Bitcoin-Qt.app -target /

    # curl -o bitcoin-22.0-osx64.tar.gz https://bitcoincore.org/bin/bitcoin-core-22.0/bitcoin-22.0-osx64.tar.gz
    # tar xzf bitcoin-22.0-osx64.tar.gz

    curl -o bitcoin.rb https://raw.githubusercontent.com/Homebrew/homebrew-core/fa6b4765d81016166f6de2bdad96cfe914c1439f/Formula/bitcoin.rb
    brew install ./bitcoin.rb
else
    curl -o bitcoin-22.0-x86_64-linux-gnu.tar.gz https://bitcoincore.org/bin/bitcoin-core-22.0/bitcoin-22.0-x86_64-linux-gnu.tar.gz
    tar xzf bitcoin-22.0-x86_64-linux-gnu.tar.gz
    sudo install -m 0755 -o root -g root -t /usr/local/bin bitcoin-22.0/bin/*
fi
