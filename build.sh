cargo build --release
strip ./target/release/argonfand
sudo cp ./target/release/argonfand /usr/bin
sudo cp ./argonfand.service /usr/lib/systemd/system/argonfand.service
