use std::fs::{create_dir_all, File};
use std::io::{Result, Write};
use std::path::Path;

/// This utility writes out a .NET 8 csproj and README.md for the
/// PhoenixAirCleanupSimulator repository, mirroring the structure and
/// physics context used for Teslaswarm-aligned Phoenix air-cleanup work.
fn main() -> Result<()> {
    let root = Path::new("PhoenixAirCleanupSimulator");
    let models_dir = root.join("Models");

    create_dir_all(&models_dir)?;

    write_csproj(&root.join("PhoenixAirCleanupSimulator.csproj"))?;
    write_readme(&root.join("README.md"))?;

    Ok(())
}

fn write_csproj(path: &Path) -> Result<()> {
    let mut file = File::create(path)?;
    let contents = r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
  </PropertyGroup>

  <ItemGroup>
    <Compile Include="Program.cs" />
    <Compile Include="Models/TreeSequestration.cs" />
    <Compile Include="Models/FilterRemoval.cs" />
    <Compile Include="Models/DACCapture.cs" />
  </ItemGroup>

  <ItemGroup>
    <None Include="Data/PhoenixAirCleanupNodes2026v1.csv">
      <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
    </None>
  </ItemGroup>

</Project>
"#;
    file.write_all(contents.as_bytes())
}

fn write_readme(path: &Path) -> Result<()> {
    let mut file = File::create(path)?;
    let contents = r#"# PhoenixAirCleanupSimulator

PhoenixAirCleanupSimulator is a .NET 8 console application that models real-world air-cleanup interventions for Phoenix, Arizona, using production-grade physics and infrastructure assumptions consistent with Teslaswarm nanoswarm and DAC models.

It focuses on:

- Tree sequestration and PM2.5 deposition along major corridors
- Electrostatic and mechanical filtration on solar-powered urban nodes
- MOF/DAC-based CO2 capture aligned with the Mesa DAC manufacturing facility

All nodes are defined in a qpudatashard CSV keyed to Phoenix geography and air-quality context.

## Repository Structure

```text
PhoenixAirCleanupSimulator/
├── PhoenixAirCleanupSimulator.csproj
├── Program.cs
├── Models/
│   ├── TreeSequestration.cs
│   ├── FilterRemoval.cs
│   └── DACCapture.cs
├── Data/
│   └── PhoenixAirCleanupNodes2026v1.csv
└── .github/
    └── workflows/
        └── ci.yml
