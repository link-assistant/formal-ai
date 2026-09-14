# Why a one-line release must not set the self-hosting bar

A release cycle that changed a single line measured 100% self-authored,
because that one line happened to be Formal AI's. A percentage measured
over one line is not evidence of a sustained share -- it is the arithmetic
of a sample too small to describe anything.

## The ledger rows

| tag | changed lines | share | trailing |
|---|---:|---:|---:|
| v0.348.0 | 49484 | 0.00% | 0 |
| v0.348.1 | 1 | 100.00% | 0 |
| v0.348.2 | 102 | 1.96% | 1 |
| v0.348.3 | 0 | 0.00% | 291 |

Weighted into the trailing window, that single line carried the trailing
share to 291 basis points. The bar for the next release is computed as
max(carried target, previous trailing share), so the bar became 2.91%.

## What it cost

The cycle that followed projected 5 basis points and was refused:

    self-hosting target would fall from 2.91% to 0.05%

That cycle was large and reviewed. It was refused for containing a great
deal of human work, which is the opposite of what the bar exists to
encourage. A bar set by a release too small to measure punished the next
release for being real.

## The rule

A share measured over fewer than 100 changed lines may not raise the
ratchet. Such a row is still recorded and still reported; it simply may
not set the floor that every later cycle has to clear. The same share
measured over a real cycle ratchets exactly as before, so this is a floor
on evidence rather than a way out of the ratchet.