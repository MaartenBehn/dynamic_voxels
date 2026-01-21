
public static bool DEBUG_PRINT = false;

public void debug<each T>(NativeString format, T args) {
  if (DEBUG_PRINT) {
    printf(format, args);
  }
}
