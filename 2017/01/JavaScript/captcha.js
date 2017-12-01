#!/usr/bin/env js

"use strict";

let r = require('readline').createInterface({input: process.stdin, output: process.stdout})

r.question("", function(inp) {
    console.log("P1: ",inp.split('').reduce((s, v, i, a) => parseInt(s) + ( (v == a[(i+1) % a.length]) ? parseInt(v) : 0 ),0));
    console.log("P2: ",inp.split('').reduce((s, v, i, a) => parseInt(s) + ( (v == a[(i+a.length/2) % a.length]) ? parseInt(v) : 0 ),0));
    r.close();
});

/*****************************
* Author: Rahul Butani       *
* Date:   December 1st, 2017 *
*****************************/