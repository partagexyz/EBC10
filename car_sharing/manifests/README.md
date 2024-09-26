# Package test using Transaction Manifest

Since our project does not have any associated back and front end yet, we'll present you in this file how the packages were tested on the Stokenet network using Radix Transaction Manifests.

## 0 Deploy the package

The package was deployed in [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_1t82n86fnvleylhg3x9uuwtxquc3gc4qwgax9zu25qu426480jlhs72tpq0/details).

## 1 Instantiate CarSharing

We first instantiate the main `CarSharing` package using [this manifest](https://instruct-stokenet.radixbillboard.com/share#61660800303b3109e63b89f791641ff358ae2187a6bacd10b83be6a3ca345761).

This creates [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_1dxtxam2uxun5fkaxask8nygh0uz93wgdsz32zz3ghy2z0fwpkwhqhd505u/details).

## 2 Create a Car Owner account

We need to create a car owner account using [this manifest](https://instruct-stokenet.radixbillboard.com/share#722db4440a70c66baa25294f0206d005f17b9c7832dc311ca4fb28902ad30f57).

This creates [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_1r70gauxyvh52fs7j4n5e9423mezat58jwrl2t2rz8ykdu0unqt5s90lp0c/details).

## 3 Add a car

We can then add a car to the platform using [this manifest](https://instruct-stokenet.radixbillboard.com/share#e44325de6eb5f77a0d9c5d6ab1ebfa7ad86ba94b8fe2ede8bfcfd59b70fea5d3).

This creates [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_1nt0t2jmrlv3vht6vt364z0k0lxtrz7hg7cwr2g6cswsvtfsdmdgs73xayz/details).

## 4 Create a user account

We have then to create a User account using [this manifest](https://instruct-stokenet.radixbillboard.com/share#5c7dfbd7df560ee6559f1c790391bd17d9586cbc1d76349bc28138eca1e9a8e5).

This creates [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_1zxvr2usva4a8zr2g2e2cu3rtv9rt7n3gkfcc9ztn5vrndndr8lqs7n9wfl/details).

## 5 Rent a car

We can then rent a car using [this manifest](https://instruct-stokenet.radixbillboard.com/share#c0e53023e7d9deca15f925ec85b45ab7a16c9d9366cf78a059ae32c441e0fbb2).

This creates [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_16ueswc27jhnfv44cvqzcacp5s8jcfj5str4cuame5w5s7e8f6l4qhmz0ca/details).

## 6 Return a car

We can then return a car using [this manifest](https://instruct-stokenet.radixbillboard.com/share#0a0ab963cc54b819bfc8d84046807ac56648f6e4734566e561148fe38a76f308).

This creates [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_1apywmj4k9lsex3rgdrtej6ax8r2qqv2lhyehgtyxpuen8x8fakvqxv882n/details).

## 7 Withdraw the car owner vault

The car owner can then withdraw his vault using [this manifest](https://instruct-stokenet.radixbillboard.com/share#60e70d6c14b98253eebf07f3aa08ae2f81491e7c17bf95a91064e5effcfd492b).

This creates [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_1y0tdn7mkg6nayc49vjx5t0mg3ah9vw2ptnzfhsfwhmkqcen4tgjsyqtedc/details).

## 8 Collect the fees

The package owner can then collect its fees using [this manifest](https://instruct-stokenet.radixbillboard.com/share#8e7a23781a531992aec04e7dd4e5d1dacb0946cca9df8c9ceb2e421e216df34a).

This creates [this transaction](https://stokenet-dashboard.radixdlt.com/transaction/txid_tdx_2_1jvud77f5fjvtvhxdnluck28cj58ye4uv6y4gxnjqd992u5xle6us3q3arx/details).
