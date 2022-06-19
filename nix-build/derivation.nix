
            { pkgs, stdenv, buildLinguaFranca, lfPackages, lib }: 
            buildLinguaFranca {
                name = "weaver";
                version = "0.1.0";
                src = ../.;
                language = "cpp";
                mainReactor = "Main";
                buildInputs = with lfPackages; [  ];
                meta = with lib; {
                    
                };
            }
            