using Xunit;

namespace MyPackage.Tests;

/// <summary>
/// Tests for the Calculator class.
/// </summary>
public class CalculatorTests
{
    /// <summary>
    /// Tests for the Add method.
    /// </summary>
    public class AddTests
    {
        [Fact]
        public void Add_PositiveNumbers_ReturnsCorrectSum()
        {
            var result = Calculator.Add(2, 3);

            Assert.Equal(5, result);
        }

        [Fact]
        public void Add_NegativeNumbers_ReturnsCorrectSum()
        {
            var result = Calculator.Add(-1, -2);

            Assert.Equal(-3, result);
        }

        [Fact]
        public void Add_Zero_ReturnsOriginalNumber()
        {
            var result = Calculator.Add(5, 0);

            Assert.Equal(5, result);
        }

        [Fact]
        public void Add_LargeNumbers_ReturnsCorrectSum()
        {
            var result = Calculator.Add(1_000_000, 2_000_000);

            Assert.Equal(3_000_000, result);
        }

        [Theory]
        [InlineData(10, 20, 30)]
        [InlineData(-100, 50, -50)]
        [InlineData(0, 0, 0)]
        [InlineData(long.MaxValue - 1, 1, long.MaxValue)]
        public void Add_VariousInputs_ReturnsExpectedResult(long a, long b, long expected)
        {
            var result = Calculator.Add(a, b);

            Assert.Equal(expected, result);
        }
    }

    /// <summary>
    /// Tests for the Multiply method.
    /// </summary>
    public class MultiplyTests
    {
        [Fact]
        public void Multiply_PositiveNumbers_ReturnsCorrectProduct()
        {
            var result = Calculator.Multiply(2, 3);

            Assert.Equal(6, result);
        }

        [Fact]
        public void Multiply_ByZero_ReturnsZero()
        {
            var result = Calculator.Multiply(5, 0);

            Assert.Equal(0, result);
        }

        [Fact]
        public void Multiply_NegativeNumbers_ReturnsCorrectProduct()
        {
            var result = Calculator.Multiply(-2, 3);

            Assert.Equal(-6, result);
        }

        [Fact]
        public void Multiply_TwoNegatives_ReturnsPositive()
        {
            var result = Calculator.Multiply(-2, -3);

            Assert.Equal(6, result);
        }

        [Theory]
        [InlineData(10, 20, 200)]
        [InlineData(-10, -20, 200)]
        [InlineData(1_000, 1_000_000, 1_000_000_000)]
        [InlineData(1, 1, 1)]
        public void Multiply_VariousInputs_ReturnsExpectedResult(long a, long b, long expected)
        {
            var result = Calculator.Multiply(a, b);

            Assert.Equal(expected, result);
        }
    }

    /// <summary>
    /// Tests for the DelayAsync method.
    /// </summary>
    public class DelayAsyncTests
    {
        [Fact]
        public async Task DelayAsync_WaitsMinimumTime()
        {
            var start = DateTime.UtcNow;

            await Calculator.DelayAsync(0.1);

            var elapsed = DateTime.UtcNow - start;
            Assert.True(elapsed.TotalSeconds >= 0.1, $"Delay should wait at least 0.1 seconds, but waited {elapsed.TotalSeconds:F4}s");
        }

        [Fact]
        public async Task DelayAsync_ZeroDelay_CompletesQuickly()
        {
            var start = DateTime.UtcNow;

            await Calculator.DelayAsync(0.0);

            var elapsed = DateTime.UtcNow - start;
            Assert.True(elapsed.TotalSeconds < 0.1, $"Zero delay should complete quickly, but took {elapsed.TotalSeconds:F4}s");
        }

        [Fact]
        public async Task DelayAsync_CancellationToken_ThrowsWhenCancelled()
        {
            using var cts = new CancellationTokenSource();
            cts.Cancel();

            await Assert.ThrowsAsync<TaskCanceledException>(
                () => Calculator.DelayAsync(10.0, cts.Token));
        }
    }
}
