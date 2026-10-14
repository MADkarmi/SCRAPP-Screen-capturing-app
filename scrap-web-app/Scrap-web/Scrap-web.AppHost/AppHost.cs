var builder = DistributedApplication.CreateBuilder(args);

builder.AddProject<Projects.Scrap_web>("scrap-web");

builder.Build().Run();
