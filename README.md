# Drink-O-Matic API

An API written in rust to turn some pumps, relays, and a Raspberry PI into an automatic personal cocktail bar.

## About The Project

Ingredients go here

<img src="preview-images/the_hardware_1.jpg?raw=true" width="100%">

Your cup goes under there

<img src="preview-images/the_hardware_2.jpg?raw=true" width="100%">

I was watching some YouTube videos about cool things to do with embedded systems when I came across something called a "barbot". It was an arduino powered cocktail bar which inspired me to try and make something similiar at a cost that wouldn't break the bank. Thus the "Drink-O-Matic" was born. I took to Aliexpress to find some parts, slapped them together in a wooden box I found from a craft store and then wrote this for my Raspberry PI. I'm not going to explain how I put everything together here but you can find a bill of materials below.

### Bill of Materials
- 1 Raspberry PI | price varies but mine was $37.07
- 6 food grade 385 diaphragm pumps | $32.16
- 1 six channel 5v trigger relay module \w octo-coupler output | $7.36
- 10 meters of 6x9 mm transparent food grade silicone tubing | $17.85
- 1 10A 12V power supply | $15.53
- 6 1UF capacitors | $2.05
- Zip ties for style points | $3.94
- Wooden box to stuff everything in | $10.32

My Total Without Shipping: $126.28 _(Not bad eh!?)_

In essence, this queues up jobs for each pump and runs them serially in the background. It optionally stores settings for my user interface if you enable the "bff" feature because I was too lazy too create yet another repo for a back-end for front-end layer. I'm also not the best with electrical engineering so it may not be necessary to limit it to 1 active pump at a time but I didn't want to chance it and burn my house down 🤣.

## Getting Started

### Prerequisites

This utilizes [GPIO character device
ABI](https://www.kernel.org/doc/Documentation/ABI/testing/gpio-cdev), ensure your Linux kernel is version >= v4.4

### Usage

1. Create a folder named ".drink-o-matic" in your user home directory ([locations by OS here](https://docs.rs/dirs/latest/dirs/fn.home_dir.html))
2. Copy the [example dotenv file](/resources/example.env) to the folder created in the above step and rename it to ".env"
   1. My relay was inverted so make double sure you set IS_RELAY_INVERTED to 0 if yours isn't or you'll have a wet floor when it turns on
   2. Additionally spend some time tweaking the MILLISECONDS_PER_ML variable once everything is setup to ensure measurements are accurate
3. Copy the [strings xml file](/resources/strings.xml) to the folder created in step 1
4. Update the ".env" to support your current configuration
5. If desired, set the address in the [rocket toml file](/Rocket.toml) to "0.0.0.0" so that other machines on your network can access the API
6. Run the following to start the API

`cargo run -r`

Note: If you are running this **for use with the frontend I made** ([umbreon222/drink-o-matic-web-interface](https://github.com/umbreon222/drink-o-matic-web-interface)), use this command instead to enable the back-end for front-end feature:

`cargo run -r --features bff`

## Development Note

I built most of this from my Windows PC which obviously doesn't support the GPIO character device
ABI. For situations like this, you can build/run with the `--no-default-features` flag. This turns off the dependency on gpio-cdev and mocks it instead which is useful for debugging and testing.

An example from when I was testing the UI:

`cargo build --no-default-features --features bff`

## License

Distributed under the MIT License. See [LICENSE.txt](/LICENSE.txt) for more information. ([back to top](/README.md#drink-o-matic-web-interface))
