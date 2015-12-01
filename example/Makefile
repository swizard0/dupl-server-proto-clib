CC = cc
CXX = c++
RM = rm -f
CPPFLAGS = -g -I ../include
LDFLAGS = -g
LDLIBS = -L ../target/release -ldupl_server_proto_clib -lzmq

SRCS = dupl_client.cpp
OBJS = $(subst .cpp,.o,$(SRCS))

all: dupl_client

dupl_client: $(OBJS)
	$(CXX) $(LDFLAGS) -o dupl_client $(OBJS) $(LDLIBS)

clean:
	$(RM) $(OBJS)

dist-clean: clean
	$(RM) dupl_client
