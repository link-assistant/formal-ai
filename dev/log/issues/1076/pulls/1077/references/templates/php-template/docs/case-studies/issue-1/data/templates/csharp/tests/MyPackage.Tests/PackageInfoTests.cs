using Xunit;

namespace MyPackage.Tests;

/// <summary>
/// Tests for the PackageInfo class.
/// </summary>
public class PackageInfoTests
{
    [Fact]
    public void Version_IsNotEmpty()
    {
        var version = PackageInfo.Version;

        Assert.NotEmpty(version);
    }

    [Fact]
    public void Version_MatchesExpectedFormat()
    {
        var version = PackageInfo.Version;

        // Version should match semantic versioning format
        Assert.Matches(@"^\d+\.\d+\.\d+", version);
    }
}
