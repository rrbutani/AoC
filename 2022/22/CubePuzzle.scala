import scala.collection.immutable.{HashSet, Queue}
// scalastyle:off

trait TraverseOn {
  type Coordinate

  def current() : Coordinate

  def setCurrent(coord : Coordinate) : Unit

  def moveLeft(amount : Double) : Coordinate
  def moveRight(amount : Double) : Coordinate
  def moveUp(amount : Double) : Coordinate
  def moveDown(amount : Double) : Coordinate

}

trait CubeFace {
  def left : CubeFace
  def right : CubeFace
  def top : CubeFace
  def bottom : CubeFace
}

case class Cube(size : Double) extends TraverseOn {

  import Cube._

  case class Coordinate(x : Double, y : Double, cubeFace : CubeFace)

  private var _current : Coordinate = Coordinate(0.0, 0.0, FRONT)

  override def current(): Coordinate = _current

  override def setCurrent(coord : Coordinate) : Unit = {
    _current = coord
  }

  override def moveLeft(amount: Double): Coordinate = {
    var newX = _current.x - amount
    var newFace = _current.cubeFace

    if ( newX < 0) {
      newX = -newX
      newFace = newFace.left
    }
    _current = Coordinate(newX, _current.y, newFace)
    _current
  }

  override def moveRight(amount: Double): Coordinate = {
    var newX = _current.x + amount
    var newFace = _current.cubeFace

    if (newX > size) {
      newX = newX - size
      newFace = newFace.right
    }

    _current = Coordinate(newX, _current.y, newFace)
    _current
  }

  override def moveUp(amount: Double): Coordinate = {
    var newY = _current.y + amount
    var newFace = _current.cubeFace

    if (newY > size) {
      newY = newY - size
      newFace = newFace.top
    }

    _current = Coordinate(_current.x, newY, newFace)
    _current
  }

  override def moveDown(amount: Double): Coordinate = {
    var newY = _current.y - amount
    var newFace = _current.cubeFace

    if (newY < 0) {
      newY = -newY
      newFace = newFace.bottom
    }

    _current = Coordinate(_current.x, newY, newFace)
    _current
  }
}

object Cube {

  object FRONT extends CubeFace {
    val left: CubeFace = LEFT
    val right: CubeFace = RIGHT
    val top: CubeFace = TOP
    val bottom: CubeFace = BOTTOM
  }

  object BACK extends CubeFace {
    val left: CubeFace = RIGHT
    val right: CubeFace = LEFT
    val top: CubeFace = TOP
    val bottom: CubeFace = BOTTOM
  }

  object TOP extends CubeFace {
    val left: CubeFace = LEFT
    val right: CubeFace = RIGHT
    val top: CubeFace = BACK
    val bottom: CubeFace = FRONT
  }

  object BOTTOM extends CubeFace {
    val left: CubeFace = LEFT
    val right: CubeFace = RIGHT
    val top: CubeFace = FRONT
    val bottom: CubeFace = BACK
  }

  object LEFT extends CubeFace {
    val left: CubeFace = BACK
    val right: CubeFace = FRONT
    val top: CubeFace = TOP
    val bottom: CubeFace = BOTTOM
  }

  object RIGHT extends CubeFace {
    val left: CubeFace = FRONT
    val right: CubeFace = BACK
    val top: CubeFace = TOP
    val bottom: CubeFace = BOTTOM
  }

}


case class UnFoldFace(var left : Option[UnFoldFace],
                      var right : Option[UnFoldFace],
                      var top : Option[UnFoldFace],
                      var bottom : Option[UnFoldFace])

case class CubeUnfold(faces : Array[UnFoldFace],
                      size : Double) extends TraverseOn {

  println(1.1)

  val cube = Cube(size)

  println(1.11)
  private val faceToIndex : Map[UnFoldFace, Int] = faces.zipWithIndex.toMap
  println(1.111)

  val cubeMapping : Array[CubeFace] = {
    println(1.1111)

    /*
     * Start with:
     * - arbitrarily assume first Face maps to Cube Front
     * - all other Faces can be any Cube Face except Front
     *
     * Then iterate over UnFoldFace relations to eliminate possibilities.
     * Assumption, for a valid Unfolding: at end of single iteration
     * we will discover a bijection from UnfoldFace - CubeFace.
     * - why: by property of a valid Unfolding
     *
     */

    val possibleCubeFace : Array[HashSet[CubeFace]] = Array.fill[HashSet[CubeFace]](6)(new HashSet[CubeFace])

    possibleCubeFace(0) += Cube.FRONT

    for(i <- 1 to 5) {
      possibleCubeFace(i) += Cube.LEFT
      possibleCubeFace(i) += Cube.RIGHT
      possibleCubeFace(i) += Cube.TOP
      possibleCubeFace(i) += Cube.BOTTOM
    }

    var queue = Queue[UnFoldFace](faces(0))

    while(queue.nonEmpty) {
      val face = queue.head
      val fI = faceToIndex(face)
      queue = queue.tail

      val cubeFace = possibleCubeFace(fI).head

      face.left.foreach{ lF =>
        val lFI = faceToIndex(lF)
        val currentCubeFaces = possibleCubeFace(lFI)
        if (currentCubeFaces.contains(cubeFace.left)) {
          if (currentCubeFaces.size > 1) {
            possibleCubeFace(lFI) = HashSet(cubeFace.left)
            queue = queue :+ lF
          }
        } else {
          throw new IllegalStateException("Assumptions are wrong")
        }
      }

      face.right.foreach { rF =>
        val rFI = faceToIndex(rF)
        val currentCubeFaces = possibleCubeFace(rFI)
        if (currentCubeFaces.contains(cubeFace.right)) {
          if (currentCubeFaces.size > 1) {
            possibleCubeFace(rFI) = HashSet(cubeFace.right)
            queue = queue :+ rF
          }
        } else {
          throw new IllegalStateException("Assumptions are wrong")
        }
      }

      face.top.foreach { tF =>
        val tFI = faceToIndex(tF)
        val currentCubeFaces = possibleCubeFace(tFI)
        if (currentCubeFaces.contains(cubeFace.top)) {
          if (currentCubeFaces.size > 1) {
            possibleCubeFace(tFI) = HashSet(cubeFace.top)
            queue = queue :+ tF
          }
        } else {
          throw new IllegalStateException("Assumptions are wrong")
        }
      }

      face.bottom.foreach { bF =>
        val bFI = faceToIndex(bF)
        val currentCubeFaces = possibleCubeFace(bFI)
        if (currentCubeFaces.contains(cubeFace.bottom)) {
          if (currentCubeFaces.size > 1) {
            possibleCubeFace(bFI) = HashSet(cubeFace.bottom)
            queue = queue :+ bF
          }
        } else {
          throw new IllegalStateException("Assumptions are wrong")
        }
      }

    }

    possibleCubeFace.map(h => h.head)
  }

  val cubeToUnfoldMap : Map[CubeFace, UnFoldFace] =
    cubeMapping.zipWithIndex.map(t => t._1 -> faces(t._2)).toMap

  cube.setCurrent(cube.Coordinate(0.0, 0.0, cubeMapping(0)))

  private def toUnfoldCoord(cCord : cube.Coordinate) : Coordinate = {
    Coordinate(cCord.x, cCord.y, cubeToUnfoldMap(cCord.cubeFace))
  }

  case class Coordinate(x : Double, y : Double, face : UnFoldFace)

  override def current(): Coordinate = toUnfoldCoord(cube.current())

  override def setCurrent(coord: Coordinate): Unit = ???

  override def moveLeft(amount: Double): Coordinate = toUnfoldCoord(cube.moveLeft(amount))

  override def moveRight(amount: Double): Coordinate = toUnfoldCoord(cube.moveRight(amount))

  override def moveUp(amount: Double): Coordinate = toUnfoldCoord(cube.moveUp(amount))

  override def moveDown(amount: Double): Coordinate = toUnfoldCoord(cube.moveDown(amount))
}

// case class UnFoldFace(left : Option[UnFoldFace],
//                       right : Option[UnFoldFace],
//                       top : Option[UnFoldFace],
//                       bottom : Option[UnFoldFace])

// case class CubeUnfold(faces : Array[UnFoldFace],
//                       size : Double) extends TraverseOn {


object App {
  def main(arr : Array[String]): Unit = {
    val arr = Array.fill[UnFoldFace](6)(new UnFoldFace(None, None, None, None));

    /* bindings, for convenience */
    val face1 = arr(0);
    val face2 = arr(1);
    val face3 = arr(2);
    val face4 = arr(3);
    val face5 = arr(4);
    val face6 = arr(5);

    // The ten starting connections:
    // face1.bottom = Some(face4);

    // face2.right  = Some(face3);

    // face3.left   = Some(face2);
    // face3.right  = Some(face4);

    // face4.top    = Some(face1);
    // face4.left   = Some(face3);
    // face4.bottom = Some(face5);

    // face5.top    = Some(face4);
    // face5.right  = Some(face6);

    // face6.left   = Some(face5);

    // Recurses infinitely, as we'd expect:
    println(arr.mkString(" "));

    println(1)
    val cube = CubeUnfold(arr, 4.0);
    println(2)

    println("hello");
    println(cube.current());
    cube.moveDown(2);
    println(cube.current());
  }
}
