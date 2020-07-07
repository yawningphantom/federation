# WASM bridge prototype

## How to make this work

```shell script
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
cd wasm-bridge
wasm-pack build -t nodejs
```


## Development with the apollo-platform-devkit

We tested this with the `apollo-platform-devkit` repo:
```shell script
git clone git@github.com:apollographql/apollo-platform-devkit.git
cd apollo-platform-devkit
git checkout abernix/apollo-rust
npm install
cd packages/rust
git checkout ran/wasm_bridge
cd ../apollo-server/
git checkout ran/get_query_plan_via_wasm
cd ../../
npm start
```

Then we just shoved a console.log in an easy test to see everything worked in the console:

```shell script
cd packages/apollo-server
npm test -- buildService
```
