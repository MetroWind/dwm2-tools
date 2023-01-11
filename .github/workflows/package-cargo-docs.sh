#!/bin/bash

mkdir -pv page-root/docs
mv target/doc/* page-root/docs/
cat << EOF > page-root/index.html
<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <title>DWM Tools Doc</title>
  </head>
  <body>
  <p><a href="docs">API doc</a></p>
  </body>
</html>
EOF

cat << EOF > page-root/docs/index.html
<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <title>DWM Tools Doc</title>
  </head>
  <body>
  <ul>
EOF

for d in $(find page-root/docs -maxdepth 1 -type d | grep -v -E '(implementors|src|docs)'); do
    DIR="$(basename "$d")"
    cat << EOF >> page-root/docs/index.html
<li><a href="${DIR}">${DIR}</a></li>
EOF
done

cat << EOF >> page-root/docs/index.html
</ul>
</body>
</html>
EOF
