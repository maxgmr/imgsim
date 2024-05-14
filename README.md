# imgsim

A program that uses various algorithms and techniques to find images that are visually similar to each other.

## Pixeldist Algorithm Options

### Euclidean

Standard Euclidean distance between two pixels' sRGB values.

### Redmean

Euclidean distance scaled to better approximate human colour perception.

## Clustering Algorithm Options

### Agglomerative

Merges neighbouring clusters together iff their pixels have a distance less than a given nth-percentile distance.
