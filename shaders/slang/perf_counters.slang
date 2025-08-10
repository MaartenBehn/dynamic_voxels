
public static bool COUNT_ITERS = true;

public static int ITERS = 0;

public void add_iter_count(int ammount) {
  if (COUNT_ITERS) {
    ITERS += ammount;
  }
}

