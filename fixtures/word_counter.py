#!/usr/bin/env python3
"""
A simple word counter script that reads a text file and counts the number of words.
"""

import sys
import argparse
from pathlib import Path


def count_words(file_path):
    """
    Count the number of words in a text file.

    Args:
        file_path: Path to the text file

    Returns:
        The number of words in the file
    """
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()
            words = content.split()
            return len(words)
    except FileNotFoundError:
        print(f"Error: File '{file_path}' not found.")
        sys.exit(1)
    except Exception as e:
        print(f"Error reading file: {e}")
        sys.exit(1)


def main():
    """Main function to parse arguments and count words."""
    parser = argparse.ArgumentParser(
        description="Count the number of words in a text file."
    )
    parser.add_argument("file", type=str, help="Path to the text file to analyze")
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="Show detailed information"
    )

    args = parser.parse_args()

    file_path = Path(args.file)
    word_count = count_words(file_path)

    if args.verbose:
        print(f"File: {file_path}")
        print(f"Word count: {word_count}")
    else:
        print(f"{word_count}")


if __name__ == "__main__":
    main()
