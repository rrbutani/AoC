#!/usr/bin/env runhaskell

import Data.List

main = do
    g <- readFile("input")
    let l = [ [ read w :: Int | w <- h] | l <- lines g, let h = words l]
    
    print( sum( [ maximum x - minimum x | x <- l ] ) )
    print( sum (concat [ [ maximum x `div` minimum x | x <- r, maximum x `mod` minimum x == 0 ] | r <- [ [ x:y:[] | (x:xs) <- tails ls, y:_ <- tails xs ] | ls <- l]]))

--------------------------------
-- Author: Rahul Butani       --
-- Date:   December 2nd, 2017 --
--------------------------------
