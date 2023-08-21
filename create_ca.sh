CANAME=${1:-"MyOrg-Root-CA"}
SUBJECT='/CN=MyOrg Root CA/C=AT/ST=Vienna/L=Vienna/O=MyOrg'

echo "CANAME: $CANAME"
echo "SUBJ: $SUBJECT"

# optional
mkdir $CANAME
cd $CANAME
# generate aes encrypted private key
openssl genrsa -aes256 -out $CANAME.key 4096
# create certificate, 1826 days = 5 years

# # the following will ask for common name, country, ...
# openssl req -x509 -new -nodes -key $CANAME.key -sha256 -days 1826 -out $CANAME.crt

# ... or you provide common name, country etc. via:
openssl req -x509 -new -nodes -key $CANAME.key -sha256 -days 1826 -out $CANAME.crt -subj "$SUBJECT"
