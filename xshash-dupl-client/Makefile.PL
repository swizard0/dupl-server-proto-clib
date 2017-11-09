use 5.008002;

use FindBin;
use ExtUtils::MakeMaker;

WriteMakefile(
    ABSTRACT       => 'HashDuplClient.pm',
    VERSION_FROM   => 'HashDuplClient.pm',
    NAME           => 'AIP::Kribrum::HashDuplClient',
    PREREQ_PM      => {  },
    AUTHOR         => 'Alexander Borisov <lex.borisov@gmail.com>',
    LIBS           => ["-L". $FindBin::Bin ."/../target/release -ldupl_server_proto_clib -lzmq"],
    DEFINE         => '',
    LINKTYPE       => 'dynamic',
    INC            => '-I'.$FindBin::Bin."/../"
);
