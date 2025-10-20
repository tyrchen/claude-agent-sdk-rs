"""Math utility functions."""


def factorial(n):
    """
    Calculate the factorial of a non-negative integer.

    Args:
        n: A non-negative integer

    Returns:
        The factorial of n (n!)

    Raises:
        ValueError: If n is negative
        TypeError: If n is not an integer
    """
    if not isinstance(n, int):
        raise TypeError(
            f"factorial() argument must be an integer, not {type(n).__name__}"
        )

    if n < 0:
        raise ValueError("factorial() not defined for negative values")

    if n == 0 or n == 1:
        return 1

    result = 1
    for i in range(2, n + 1):
        result *= i

    return result
