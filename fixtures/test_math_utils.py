"""Tests for math_utils module."""

import unittest
from math_utils import factorial


class TestFactorial(unittest.TestCase):
    """Test cases for the factorial function."""

    def test_factorial_zero(self):
        """Test factorial of 0."""
        self.assertEqual(factorial(0), 1)

    def test_factorial_one(self):
        """Test factorial of 1."""
        self.assertEqual(factorial(1), 1)

    def test_factorial_positive_small(self):
        """Test factorial of small positive integers."""
        self.assertEqual(factorial(5), 120)
        self.assertEqual(factorial(3), 6)
        self.assertEqual(factorial(4), 24)

    def test_factorial_positive_large(self):
        """Test factorial of larger positive integers."""
        self.assertEqual(factorial(10), 3628800)

    def test_factorial_negative(self):
        """Test that factorial raises ValueError for negative numbers."""
        with self.assertRaises(ValueError):
            factorial(-1)
        with self.assertRaises(ValueError):
            factorial(-5)

    def test_factorial_non_integer(self):
        """Test that factorial raises TypeError for non-integer inputs."""
        with self.assertRaises(TypeError):
            factorial(3.5)
        with self.assertRaises(TypeError):
            factorial("5")
        with self.assertRaises(TypeError):
            factorial(None)


if __name__ == "__main__":
    unittest.main()
