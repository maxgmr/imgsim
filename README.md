# imgsim

A program that uses various algorithms and techniques to find images that are visually similar to each other.

Currently only works on Linux machines.

## Usage

### Command

`imgsim [OPTIONS] [input_dir]`

### Arguments

`[input_dir]`: The path to the directory of images you wish to compare. Selects the current working directory by default.

### Options

Not all algorithms are usable with all other algorithms. Kindly view the algorithm options in the following sections to check for any such restrictions.

- `-p, --pixeldist <pixeldist_algorithm>`: Choose the algorithm for pixel distance
- `-c, --clustering <clustering_algorithm>`: Choose the algorithm for pixel clustering
- `-s, --similarity <similarity_algorithm>`: Choose the algorithm for image similarity
- `-v, --verbose`: Print more messages to the terminal.
- `-o, --output <output_dir>`: The directory to which debug images are saved. Leave this blank to not save any debug images.
- `-h, --help`: Print help
- `-V, --version`: Print version

## Pixeldist Algorithm Options

### Euclidean

Standard Euclidean distance between two pixels' sRGB values.

### Redmean

Euclidean distance scaled to better approximate human colour perception.

## Clustering Algorithm Options

### Agglomerative

Merges neighbouring clusters together iff their pixels have a distance less than a given nth-percentile distance.

## Similarity Algorithm Options

### Coloursim

Calculates image similarity based on the average colours of the most dominant clusters of each image.

### Clustersize

Calculates image similarity based on the relative size and relative location of each image's most dominant clusters.
