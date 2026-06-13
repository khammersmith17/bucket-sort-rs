# Naive Bucket Sort
This is just a toy naive implementation of a distributed bucket sort algorithm that I spent an afternoon playing with. I implemented something like this in school using MPI and had an idea of how to create the MPI protocol in rust. This is just a very basic implementation of the idea, using distribtuted bucket sort as the algorithm of choice to try it on.

Also implemented a barrier type.

Very naive, it will almost certainly perform worse that regular sort on small to medium size datasets and does not cover edge small edges cases (assumes relative uniform distributions).

A basic benchmark on 10 million i32 integers clocks the disrtibuted bucket sort at ~2 seconds and built in sort at ~5 seconds, of course with paying the cost of copying the entire Vec.
