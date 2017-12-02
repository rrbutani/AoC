val input = scala.io.StdIn.readLine().split("").map(_.toInt)

println(s"P1: ${(input :+ input.head).sliding(2).toList.foldLeft(0)((s, t) => (if (t(0) == t(1)) t(0) + s else s))}") 
println(s"P2: ${(input zip (input.drop(input.length/2) ++ input.take(input.length/2))).foldLeft(0)((s, t) => if (t._1 == t._2) t._1 + s else s)}")

/*****************************
* Author: Rahul Butani       *
* Date:   December 2nd, 2017 *
*****************************/
