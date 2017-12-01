import Data.Char

main = do
    l <- getLine
    print (sum [ digitToInt (fst x) | x <- zip l (drop 1            (cycle l)), fst x == snd x ])
    print (sum [ digitToInt (fst x) | x <- zip l (drop ((length l) `div` 2) (cycle l)), fst x == snd x ])
