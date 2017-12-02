import Data.List

main = do
    g <- readFile("input")
    let l = [ [ read w :: Int | w <- h] | l <- lines g, let h = words l]
    
    print( sum( [ maximum x - minimum x | x <- l ] ) )
    print( sum (concat [ [ maximum x `div` minimum x | x <- r, maximum x `mod` minimum x == 0 ] | r <- [ [ x:y:[] | (x:xs) <- tails ls, y:_ <- tails xs ] | ls <- l]]))

-- , maximum x `mod` minimum x == 0]

    -- print (map read $ words "1 2 3" :: [Int])

    -- print()

-- [x++y++z | (x:xs) <- tails list, (y:ys) <- tails xs, (z:_) <- tails ys]
-- [x++y | (x:xs) <- tails ls, (y:_) <- tails xs]

-- combinations 0 _ = [[]]
-- combinations n ls = [ (x:ys) | (x:xs) <- tails ls, ys <- combinations (n-1) xs ]

-- [ maximum x `div` minimum x | x <- [ x:y:[] | (x:xs) <- tails ls, y:_ <- tails xs ], maximum x `mod` minimum x == 0] 