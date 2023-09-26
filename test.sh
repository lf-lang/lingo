cargo b --release
git clone https://github.com/lf-lang/lingua-franca.git ./lf-test
curl -L0 https://github.com/lf-lang/lingua-franca/releases/download/v0.4.0/lf-cli-0.4.0.tar.gz --output ./lf-release.tar.gz
tar -xvf ./lf-release.tar.gz
#tar -xf ./lf-release.tar.gz
#sed -i 's/#!//bin//bash/#!//run//current-system//sw//bin//bash//' lf-cli-0.4.0/bin/lfc
cp test/Lingo-Cpp.toml ./lf-test/test/Cpp/Lingo.toml
cd ./lf-test/test/Cpp
ls ../../../
ls
../../../target/release/lingo build --lfc ../../../lf-cli-0.4.0/bin/lfc
